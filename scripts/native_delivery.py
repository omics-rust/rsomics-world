import argparse
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import tarfile
import tomllib


TARGETS = {
    "x86_64-unknown-linux-gnu": ("linux-x86_64", "ELF 64-bit", "x86-64"),
    "aarch64-unknown-linux-gnu": ("linux-aarch64", "ELF 64-bit", "aarch64"),
    "x86_64-apple-darwin": ("macos-x86_64", "Mach-O 64-bit executable", "x86_64"),
    "aarch64-apple-darwin": ("macos-aarch64", "Mach-O 64-bit executable", "arm64"),
}
NOTICE_NAME = re.compile(r"^(license|licence|copying|notice|copyright|unlicense)(?:$|[._-])", re.I)


def require(condition, message):
    if not condition:
        raise ValueError(message)


def read_command(repo, *argv):
    return subprocess.check_output(argv, cwd=repo)


def identity(file):
    with file.open("rb") as stream:
        return {"bytes": file.stat().st_size, "sha256": hashlib.file_digest(stream, "sha256").hexdigest()}


def json_body(value):
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()


def write_new(file, body):
    with file.open("xb") as stream:
        stream.write(body)


def git_bodies(repo, head):
    expected = {}
    for row in read_command(repo, "git", "ls-tree", "-rz", "--full-tree", head).split(b"\0"):
        if not row:
            continue
        entry, raw_name = row.split(b"\t", 1)
        mode, kind, blob = entry.decode().split()
        name = raw_name.decode()
        require(kind == "blob" and mode in ("100644", "100755"), f"unsupported Git entry: {name}")
        expected[name] = (int(mode, 8) & 0o777, read_command(repo, "git", "cat-file", "blob", blob))
    return expected


def archive_path(member, prefix):
    name = PurePosixPath(member.name)
    require(not name.is_absolute() and name.parts and name.parts[0] == prefix and ".." not in name.parts,
            f"unsafe archive member: {member.name}")
    return PurePosixPath(*name.parts[1:]).as_posix()


def extract_exact_package(repo, head, archive, destination):
    expected = git_bodies(repo, head)
    originals = {"Cargo.toml.orig", ".cargo_vcs_info.json"}
    bodies, seen = {}, set()
    with tarfile.open(archive, "r:gz") as package:
        for member in package:
            name = archive_path(member, destination.name)
            require(name not in seen, f"duplicate archive member: {name}")
            seen.add(name)
            require(member.isfile() or member.isdir(), f"unsupported archive member: {name}")
            if member.isfile():
                mode = expected[name][0] if name in expected else 0o644
                require(name != "." and member.mode == mode, f"source package mode differs: {name}")
                bodies[name] = package.extractfile(member).read()
    require(set(bodies) == expected.keys() | originals, "source package membership differs from Git")
    vcs = json.loads(bodies[".cargo_vcs_info.json"])
    require(vcs["git"]["sha1"] == head and not vcs["git"].get("dirty") and vcs["path_in_vcs"] == "",
            "source package does not identify the clean exact head")
    for name, (_, body) in expected.items():
        actual = "Cargo.toml.orig" if name == "Cargo.toml" else name
        require(bodies[actual] == body, f"source package body differs: {actual}")
    destination.mkdir(parents=True)
    for name, body in bodies.items():
        path = destination / name
        path.parent.mkdir(parents=True, exist_ok=True)
        write_new(path, body)
        path.chmod(expected[name][0] if name in expected else 0o644)
    return {"original_members": len(expected), "package_members": len(bodies), "archive": identity(archive)}


def cargo_metadata(repo, *options):
    return json.loads(read_command(repo, "cargo", "metadata", "--locked", "--offline", "--format-version", "1", *options))


def package_contract(package):
    parent = Path(package["manifest_path"]).parent
    result = {k: v for k, v in package.items() if k not in ("id", "manifest_path", "targets")}
    result["targets"] = [
        {**target, "src_path": str(Path(target["src_path"]).relative_to(parent))}
        for target in package["targets"]
    ]
    return result


def dependency_notices(repo, host, output):
    metadata = cargo_metadata(repo, "--filter-platform", host)
    packages = {(p["name"], p["version"]): p for p in metadata["packages"]}
    require(len(packages) == len(metadata["packages"]), "ambiguous dependency source in Cargo graph")
    lock = tomllib.loads((repo / "Cargo.lock").read_text())
    checksums = {(p["name"], p["version"], p.get("source")): p.get("checksum") for p in lock["package"]}
    tree = read_command(repo, "cargo", "tree", "--locked", "--offline", "--target", host,
                        "--edges", "normal,build", "--prefix", "none", "--format", "{p}")
    enabled = {}
    for line in tree.decode().splitlines():
        name, version = re.match(r"^(\S+) v(\S+)", line).groups()
        package = packages[name, version]
        if Path(package["manifest_path"]) != repo / "Cargo.toml":
            enabled[package["id"]] = package
    output.mkdir()
    rows = []
    for package_id, package in sorted(enabled.items()):
        name, version, source = (package[k] for k in ("name", "version", "source"))
        checksum = checksums[name, version, source]
        require(source and source.startswith("registry+") and checksum, f"unverified dependency: {package_id}")
        crate_dir = Path(package["manifest_path"]).parent
        prefix = f"{name}-{version}"
        registry_archive = crate_dir.parents[2] / "cache" / crate_dir.parent.name / f"{prefix}.crate"
        require(identity(registry_archive)["sha256"] == checksum, f"registry checksum differs: {package_id}")
        declared = package["license_file"]
        originals = {}
        with tarfile.open(registry_archive, "r:gz") as archive:
            for member in archive:
                path = archive_path(member, prefix)
                if member.isfile() and (path == declared or NOTICE_NAME.match(PurePosixPath(path).name)):
                    require(path not in originals, f"duplicate dependency notice: {prefix}/{path}")
                    destination = output / prefix / path
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    write_new(destination, archive.extractfile(member).read())
                    originals[path] = identity(destination)
        require(originals and (declared is None or declared in originals), f"missing dependency notice: {package_id}")
        rows.append({"package": package_id, "license": package["license"], "license_file": declared,
                     "registry_archive_sha256": checksum, "originals": originals})
    write_new(output / "cargo-tree.txt", tree)
    write_new(output / "INDEX.json", json_body({"native_host": host, "edges": ["normal", "build"], "packages": rows}))
    return identity(output / "INDEX.json")


def source(repo, head, host, out):
    require(read_command(repo, "git", "rev-parse", "HEAD").decode().strip() == head, "checkout is not exact head")
    require(not read_command(repo, "git", "status", "--porcelain"), "source checkout is dirty")
    compiler = read_command(repo, "rustc", "-vV").decode()
    require(f"host: {host}" in compiler.splitlines(), "compiler is not the native target")
    metadata = cargo_metadata(repo, "--no-deps")
    package, = (p for p in metadata["packages"] if Path(p["manifest_path"]) == repo / "Cargo.toml")
    prefix = f"{package['name']}-{package['version']}"
    archive = Path(metadata["target_directory"]) / "package" / f"{prefix}.crate"
    out.mkdir(parents=True, exist_ok=True)
    extracted = out / ".source" / prefix
    proof = extract_exact_package(repo, head, archive, extracted)
    normalized, = cargo_metadata(extracted, "--no-deps")["packages"]
    require(package_contract(normalized) == package_contract(package), "normalized Cargo metadata differs")
    with archive.open("rb") as src, (out / archive.name).open("xb") as dst:
        shutil.copyfileobj(src, dst)
    notices = dependency_notices(extracted, host, out / "dependency-licenses")
    receipt = {"product": package["name"], "version": package["version"], "head_sha": head,
               "native_host": host, "rustc": compiler, "lock": identity(extracted / "Cargo.lock"),
               "source_package": {**proof, "file": archive.name, "normalized_metadata_match": True},
               "dependency_notice_index": notices, "helper": identity(Path(__file__))}
    write_new(out / "SOURCE.json", json_body(receipt))
    print(extracted)


def bundle(out, binary):
    receipt = json.loads((out / "SOURCE.json").read_bytes())
    lifecycle = out / "lifecycle/VERIFIED.json"
    installed = json.loads(lifecycle.read_bytes())
    require(identity(binary) == installed["binary"], "bundle binary differs from installed lifecycle")
    require(installed["source_head"] == receipt["head_sha"] and installed["version"] == receipt["version"],
            "installed lifecycle is not for this source package")
    require(identity(Path(__file__)) == receipt["helper"], "delivery helper changed after source verification")
    source_archive = out / receipt["source_package"]["file"]
    require(identity(source_archive) == receipt["source_package"]["archive"], "source archive changed after verification")
    notices = out / "dependency-licenses"
    require(identity(notices / "INDEX.json") == receipt["dependency_notice_index"], "dependency notice index changed")
    target, file_format, cpu = TARGETS[receipt["native_host"]]
    description = read_command(out, "file", "-b", str(binary)).decode().strip()
    require(file_format in description and cpu in description, f"installed executable does not match {target}: {description}")
    linkage = ("otool", "-L") if target.startswith("macos-") else ("ldd",)
    libraries = read_command(out, *linkage, str(binary)).decode()
    require("not found" not in libraries, "installed executable has an unresolved runtime dependency")
    product = receipt["product"]
    stem = f"{product}-{receipt['version']}-{target}"
    extracted = out / ".source" / f"{product}-{receipt['version']}"
    for notice in ("LICENSE-MIT", "LICENSE-APACHE-2.0", "THIRD_PARTY_NOTICES.md"):
        require((extracted / notice).is_file(), f"missing product notice: {notice}")
    build = {**receipt, "target": target, "binary": installed["binary"], "binary_format": description,
             "dynamic_libraries": libraries,
             "lifecycle": identity(lifecycle), "run_id": os.environ.get("GITHUB_RUN_ID"),
             "run_attempt": os.environ.get("GITHUB_RUN_ATTEMPT")}
    archive = out / f"{stem}.tar.gz"
    with tarfile.open(archive, "x:gz") as carrier:
        carrier.add(binary, arcname=f"{stem}/{product}")
        for name in ("README.md", "PROVENANCE.md", "LICENSE-MIT", "LICENSE-APACHE-2.0",
                     "THIRD_PARTY_NOTICES.md", "THIRD_PARTY_LICENSES.md", "LICENSES"):
            if (extracted / name).exists():
                carrier.add(extracted / name, arcname=f"{stem}/{name}")
        carrier.add(notices, arcname=f"{stem}/dependency-licenses")
        body = json_body(build)
        member = tarfile.TarInfo(f"{stem}/BUILD.json")
        member.size = len(body)
        carrier.addfile(member, io.BytesIO(body))
    write_new(out / "SHA256SUMS", "".join(
        f"{identity(file)['sha256']}  {file.name}\n" for file in (archive, source_archive)
    ).encode())
    print(archive)


def main():
    parser = argparse.ArgumentParser()
    commands = parser.add_subparsers(dest="mode", required=True)
    prepare = commands.add_parser("source")
    prepare.add_argument("--repo", type=Path, required=True)
    prepare.add_argument("--head", required=True)
    prepare.add_argument("--host", choices=TARGETS, required=True)
    pack = commands.add_parser("bundle")
    pack.add_argument("--binary", type=Path, required=True)
    for command in (prepare, pack):
        command.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    if args.mode == "source":
        source(args.repo.resolve(strict=True), args.head, args.host, args.out.resolve())
    else:
        bundle(args.out.resolve(strict=True), args.binary.resolve(strict=True))


if __name__ == "__main__":
    main()
