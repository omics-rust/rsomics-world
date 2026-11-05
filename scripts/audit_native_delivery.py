"""Verify uploaded native delivery originals without executing their binaries."""

import argparse
import gzip
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import subprocess
import tarfile
import tomllib
import zipfile

import native_delivery
from native_delivery import (
    TARGETS, cargo_metadata, extract_exact_package, identity, json_body,
    package_contract, read_command, require, write_new,
)


MAX_ZIP = 64 << 20
MAX_UNPACKED = 128 << 20
HOSTS = {details[0]: host for host, details in TARGETS.items()}
PRODUCT_NOTICES = (
    "README.md", "PROVENANCE.md", "LICENSE-MIT", "LICENSE-APACHE-2.0",
    "THIRD_PARTY_NOTICES.md", "THIRD_PARTY_LICENSES.md",
)


def body_identity(body):
    return {"bytes": len(body), "sha256": hashlib.sha256(body).hexdigest()}


def artifact_name(product, target, head, attempt):
    return f"{product}-delivery-{target}-{head}-a{attempt}"


def artifact_identity(row):
    size, digest = row["size_in_bytes"], row["digest"]
    require(type(size) is int and 0 < size <= MAX_ZIP, "invalid original ZIP size")
    require(isinstance(digest, str) and re.fullmatch(r"sha256:[0-9a-f]{64}", digest),
            "invalid original ZIP digest")
    return {"bytes": size, "sha256": digest[7:]}


def check_artifact(row, name, head, run_id):
    require(row["name"] == name and row["expired"] is False, "incorrect or expired artifact")
    require(type(row["id"]) is int and row["id"] > 0, "invalid artifact ID")
    run = row["workflow_run"]
    require(run["id"] == run_id and run["head_sha"] == head, "artifact run or head differs")
    return artifact_identity(row)


def artifact_targets(payload, product, head, run_id, attempt):
    names = {artifact_name(product, target, head, attempt): host for target, host in HOSTS.items()}
    chosen = {}
    for row in payload["artifacts"]:
        name = row["name"]
        if name in names:
            host = names[name]
            require(host not in chosen, f"duplicate native artifact: {name}")
            check_artifact(row, name, head, run_id)
            chosen[host] = row
    require(len(chosen) == len(HOSTS) and len({row["id"] for row in chosen.values()}) == len(HOSTS),
            "expected four native artifacts for this run attempt")
    return chosen


def safe_name(name):
    path = PurePosixPath(name)
    require(name != "." and path.as_posix() == name and not path.is_absolute()
            and ".." not in path.parts and "\\" not in name and "\0" not in name,
            f"unsafe archive path: {name!r}")
    return path


def zip_files(archive):
    files, seen, total = {}, set(), 0
    with zipfile.ZipFile(archive) as bundle:
        for member in bundle.infolist():
            name = safe_name(member.filename.rstrip("/")).as_posix()
            mode = stat.S_IFMT(member.external_attr >> 16)
            require(mode in (0, stat.S_IFREG, stat.S_IFDIR), f"unsupported ZIP member: {name}")
            require(name.casefold() not in seen, f"duplicate ZIP member: {name}")
            seen.add(name.casefold())
            total += member.file_size
            require(total <= MAX_UNPACKED, "original ZIP exceeds unpacked limit")
            if not member.is_dir():
                files[name] = bundle.read(member)
    return files


def unpack_tar(body, label):
    with gzip.GzipFile(fileobj=io.BytesIO(body)) as archive:
        unpacked = archive.read(MAX_UNPACKED + 1)
    require(len(unpacked) <= MAX_UNPACKED, f"{label} exceeds unpacked limit")
    return unpacked


def tar_files(body, stem, executable):
    files, seen = {}, set()
    with tarfile.open(fileobj=io.BytesIO(unpack_tar(body, "carrier")), mode="r:") as archive:
        for member in archive:
            path = safe_name(member.name.rstrip("/"))
            require(path.parts[0] == stem and (member.isfile() or member.isdir()),
                    f"unexpected carrier member: {path}")
            name = PurePosixPath(*path.parts[1:]).as_posix()
            require(name.casefold() not in seen, f"duplicate carrier member: {path}")
            seen.add(name.casefold())
            if member.isfile():
                require(name != ".", "carrier root is not a directory")
                if name == executable:
                    require(member.mode & 0o111, "installed binary is not executable in carrier")
                files[name] = archive.extractfile(member).read()
    return files


def binary_matches(body, target):
    if target.startswith("linux-"):
        machine = 62 if target.endswith("x86_64") else 183
        return body.startswith(b"\x7fELF\x02\x01") and int.from_bytes(body[18:20], "little") == machine
    machine = 0x01000007 if target.endswith("x86_64") else 0x0100000C
    return body.startswith(b"\xcf\xfa\xed\xfe") and int.from_bytes(body[4:8], "little") == machine


def verify_notices(carrier, index_body, host, lock, root_package):
    root = "dependency-licenses/"
    index = json.loads(index_body)
    require(index["native_host"] == host and index["edges"] == ["normal", "build"],
            "dependency notice index target or edges differ")
    locked = {f"{p['source']}#{p['name']}@{p['version']}": p["checksum"]
              for p in lock["package"] if "source" in p and "checksum" in p}
    tree = set()
    for line in carrier[f"{root}cargo-tree.txt"].decode().splitlines():
        name, version = re.match(r"^(\S+) v(\S+)", line).groups()
        tree.add((name, version))
    tree.discard(root_package)
    packages, names = set(), {f"{root}INDEX.json", f"{root}cargo-tree.txt"}
    for row in index["packages"]:
        package = row["package"]
        name, version = package.rsplit("#", 1)[1].rsplit("@", 1)
        require(package in locked and locked[package] == row["registry_archive_sha256"],
                f"dependency notice is not bound to the lockfile: {name}")
        require((name, version) not in packages, f"duplicate dependency package: {name}")
        packages.add((name, version))
        originals = row["originals"]
        declared = row["license_file"]
        require(originals and (declared is None or declared in originals), f"missing notice: {name}")
        for original, expected in originals.items():
            path = safe_name(f"{root}{name}-{version}/{original}").as_posix()
            require(path not in names and path in carrier and body_identity(carrier[path]) == expected,
                    f"dependency notice body differs: {path}")
            names.add(path)
    require(packages == tree and names == {name for name in carrier if name.startswith(root)},
            "dependency notice membership differs")
    return len(names) - 2


def package_id(repo, head):
    manifest = tomllib.loads(read_command(repo, "git", "show", f"{head}:Cargo.toml").decode())
    package = manifest["package"]
    product, version = package["name"], package["version"]
    require(re.fullmatch(r"[a-zA-Z0-9_-]+", product) and re.fullmatch(r"[a-zA-Z0-9.+-]+", version),
            "invalid package identity")
    return product, version


def metadata_contract(repo):
    package, = (p for p in cargo_metadata(repo, "--no-deps")["packages"]
                if Path(p["manifest_path"]) == repo / "Cargo.toml")
    return package_contract(package)


def verify_lifecycle(files, installed, product, version):
    calls = json.loads(files["lifecycle/commands.json"])
    require(calls and calls[0]["argv"][1:] == ["--version"], "missing installed version call")
    binary = calls[0]["argv"][0]
    require(Path(binary).name == product
            and files["lifecycle/1.stdout"].strip() == f"{product} {version}".encode(),
            "installed version differs")
    require(sum(call["argv"][0] == binary for call in calls) == installed["native_calls"],
            "installed native call count differs")
    exits = [call["returncode"] for call in calls]
    require(exits[0] == 0 and all(type(status) is int and status >= 0 for status in exits)
            and sum(status != 0 for status in exits) == installed.get("expected_failures", 0),
            "unexpected installed command status")
    for i in range(1, len(calls) + 1):
        require(all(f"lifecycle/{i}.{stream}" in files for stream in ("stdout", "stderr")),
                f"missing installed command output: {i}")


def verify_archive(repo, head, host, run_id, attempt, artifact, archive, out):
    product, version = package_id(repo, head)
    target, binary_format, cpu = TARGETS[host]
    name = artifact_name(product, target, head, attempt)
    expected_zip = check_artifact(artifact, name, head, run_id)
    require(identity(archive) == expected_zip, f"original ZIP identity differs: {name}")
    files = zip_files(archive)
    prefix, stem = f"{product}-{version}", f"{product}-{version}-{target}"
    source_name, carrier_name = f"{prefix}.crate", f"{stem}.tar.gz"
    source_body, carrier_body = files[source_name], files[carrier_name]
    source_id, carrier_id = body_identity(source_body), body_identity(carrier_body)
    require(files["SHA256SUMS"] == (
        f"{carrier_id['sha256']}  {carrier_name}\n{source_id['sha256']}  {source_name}\n"
    ).encode(), "carrier/source SHA256SUMS differs")
    source = json.loads(files["SOURCE.json"])
    require((source["product"], source["version"], source["head_sha"], source["native_host"])
            == (product, version, head, host) and f"host: {host}" in source["rustc"].splitlines(),
            "source receipt identity differs")
    require(source["helper"] == identity(Path(native_delivery.__file__)), "producer helper differs")
    carrier = tar_files(carrier_body, stem, product)
    build = json.loads(carrier["BUILD.json"])
    require(all(build.get(key) == value for key, value in source.items())
            and (build["target"], build["run_id"], build["run_attempt"]) == (target, str(run_id), str(attempt)),
            "carrier source or run/attempt differs")
    require(binary_format in build["binary_format"] and cpu in build["binary_format"]
            and build["dynamic_libraries"] and "not found" not in build["dynamic_libraries"],
            "carrier native format or linkage receipt differs")
    lifecycle = files["lifecycle/VERIFIED.json"]
    installed = json.loads(lifecycle)
    binary = body_identity(carrier[product])
    require(build["lifecycle"] == body_identity(lifecycle) and installed["source_head"] == head
            and installed["version"] == version and build["binary"] == installed["binary"] == binary
            and binary_matches(carrier[product], target), "carrier differs from the installed binary")
    verify_lifecycle(files, installed, product, version)
    index = files["dependency-licenses/INDEX.json"]
    require(carrier["dependency-licenses/INDEX.json"] == index
            and source["dependency_notice_index"] == body_identity(index), "carrier notice index differs")

    unpack_tar(source_body, "source crate")
    out.mkdir()
    crate = out / source_name
    write_new(crate, source_body)
    extracted = out / prefix
    proof = extract_exact_package(repo, head, crate, extracted)
    require(source["source_package"] == {**proof, "file": source_name, "normalized_metadata_match": True},
            "exact source package receipt differs")
    require(source["lock"] == identity(extracted / "Cargo.lock"), "carrier lockfile differs")
    require(metadata_contract(extracted) == metadata_contract(repo), "normalized Cargo metadata differs")
    expected_core = {product, "BUILD.json"}
    for name in PRODUCT_NOTICES:
        path = extracted / name
        if path.is_file():
            require(carrier.get(name) == path.read_bytes(), f"product notice differs: {name}")
            expected_core.add(name)
    for path in (extracted / "LICENSES").rglob("*"):
        if path.is_file():
            name = path.relative_to(extracted).as_posix()
            require(carrier.get(name) == path.read_bytes(), f"product notice differs: {name}")
            expected_core.add(name)
    require(expected_core == {name for name in carrier if not name.startswith("dependency-licenses/")},
            "carrier membership differs from the product")
    lock = tomllib.loads((extracted / "Cargo.lock").read_text())
    count = verify_notices(carrier, index, host, lock, (product, version))
    return {"artifact_id": artifact["id"], "artifact_name": artifact["name"], "zip": expected_zip,
            "source": source_id, "carrier": carrier_id, "binary": binary,
            "source_members": proof["original_members"], "notice_bodies": count,
            "native_calls": installed["native_calls"], "lifecycle": body_identity(lifecycle)}


def fetch(path, token, *, stream=subprocess.PIPE, limit=MAX_ZIP):
    result = subprocess.run(
        ["curl", "-q", "--fail", "--silent", "--show-error", "--location", "--max-redirs", "2",
         "--proto", "=https", "--proto-redir", "=https", "--connect-timeout", "10", "--max-time", "120",
         "--max-filesize", str(limit), "--header", "@-", f"https://api.github.com/{path}"],
        input=f"Authorization: Bearer {token}\nAccept: application/vnd.github+json\n".encode(),
        stdout=stream, check=True, timeout=125,
    )
    return result.stdout


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    repo = args.repo.resolve(strict=True)
    token = os.environ.pop("ARTIFACT_TOKEN")
    head = os.environ["GITHUB_SHA"]
    require(re.fullmatch(r"[0-9a-f]{40}", head) and read_command(repo, "git", "rev-parse", "HEAD").decode().strip() == head,
            "audit checkout is not the exact head")
    run_id, attempt = int(os.environ["GITHUB_RUN_ID"]), int(os.environ["GITHUB_RUN_ATTEMPT"])
    require(run_id > 0 and attempt > 0 and token and "\r" not in token and "\n" not in token,
            "missing or invalid same-run audit context")
    product, version = package_id(repo, head)
    repository = os.environ["GITHUB_REPOSITORY"]
    require(repository == f"omics-rust/{product}", "audit is limited to the owning repository")
    route = f"repos/{repository}/actions"
    out = args.out.resolve()
    out.mkdir()
    listing = fetch(f"{route}/runs/{run_id}/artifacts?per_page=100", token, limit=1 << 20)
    write_new(out / "ARTIFACTS.json", listing)
    artifacts = artifact_targets(json.loads(listing), product, head, run_id, attempt)
    targets = {}
    for target, host in sorted(HOSTS.items()):
        row = artifacts[host]
        archive = out / f"{target}.zip"
        with archive.open("xb") as stream:
            fetch(f"{route}/artifacts/{row['id']}/zip", token, stream=stream, limit=row["size_in_bytes"])
        targets[target] = verify_archive(repo, head, host, run_id, attempt, row, archive, out / target)
    require(len({row["source"]["sha256"] for row in targets.values()}) == 1,
            "native jobs did not package identical source archives")
    receipt = {"schema": "rsomics.native-original-audit.v1", "repository": repository,
               "product": product, "version": version, "head": head, "run_id": run_id, "attempt": attempt,
               "targets": targets, "release_authorized": False}
    audit = out / "AUDITED.json"
    write_new(audit, json_body(receipt))
    print(f"Verified four original delivery artifacts for {product} {version}, head {head}, run {run_id} attempt {attempt}.")
    for target, row in targets.items():
        print(f"- {target}: artifact {row['artifact_id']}, ZIP SHA-256 {row['zip']['sha256']}")
    print(f"Audit receipt SHA-256: {identity(audit)['sha256']}")


if __name__ == "__main__":
    main()
