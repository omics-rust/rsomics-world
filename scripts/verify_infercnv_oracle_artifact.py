"""Offline verification and extraction of one pinned inferCNV Actions artifact."""

import argparse
import hashlib
import json
import os
import re
import stat
import sys
import unicodedata
import zipfile
import zlib
from pathlib import Path


REPOSITORY = "omics-rust/rsomics-world"
WORKFLOW = ".github/workflows/infercnv-oracle.yml"
BUNDLE = "shipped-bundle"
MANIFEST = "sha256-manifest.tsv"
HEX40 = re.compile(r"[0-9a-f]{40}\Z")
HEX64 = re.compile(r"[0-9a-f]{64}\Z")
RECEIPT_FIELDS = {
    "schema_version", "repository", "workflow_path", "run_id", "run_attempt",
    "artifact_id", "head_sha", "artifact_name", "artifact_sha256",
    "manifest_sha256", "bundle_path",
}


def _object_pairs(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON field: {key}")
        result[key] = value
    return result


def _load_json(path, label):
    try:
        with Path(path).open("r", encoding="utf-8") as source:
            value = json.load(source, object_pairs_hook=_object_pairs)
    except (UnicodeError, json.JSONDecodeError) as exc:
        raise ValueError(f"invalid {label} JSON: {exc}") from exc
    if not isinstance(value, dict):
        raise ValueError(f"{label} must be a JSON object")
    return value


def _positive_int(value):
    return type(value) is int and value > 0


def _matches(value, pattern):
    return isinstance(value, str) and pattern.fullmatch(value) is not None


def _receipt(path):
    receipt = _load_json(path, "receipt")
    if set(receipt) != RECEIPT_FIELDS:
        raise ValueError("receipt fields do not match schema")
    if type(receipt["schema_version"]) is not int or receipt["schema_version"] != 1:
        raise ValueError("unsupported receipt schema_version")
    for field in ("run_id", "run_attempt", "artifact_id"):
        if not _positive_int(receipt[field]):
            raise ValueError(f"invalid receipt {field}")
    for field, pattern in (("head_sha", HEX40), ("artifact_sha256", HEX64), ("manifest_sha256", HEX64)):
        if not _matches(receipt[field], pattern):
            raise ValueError(f"invalid receipt {field}")
    expected = {
        "repository": REPOSITORY,
        "workflow_path": WORKFLOW,
        "bundle_path": BUNDLE,
        "artifact_name": f"infercnv-oracle-{receipt['run_id']}-{receipt['run_attempt']}",
    }
    for field, value in expected.items():
        if receipt[field] != value:
            raise ValueError(f"invalid receipt {field}")
    return receipt


def _run(path, receipt):
    run = _load_json(path, "run")
    expected = {
        "id": receipt["run_id"], "run_attempt": receipt["run_attempt"],
        "head_sha": receipt["head_sha"], "path": receipt["workflow_path"],
        "status": "completed", "conclusion": "success", "event": "workflow_dispatch",
    }
    for field, value in expected.items():
        if type(run.get(field)) is not type(value) or run[field] != value:
            raise ValueError(f"run {field} does not match receipt or required status")
    for field in ("repository", "head_repository"):
        repository = run.get(field)
        if not isinstance(repository, dict) or repository.get("full_name") != receipt["repository"]:
            raise ValueError(f"run {field} does not match receipt")


def _artifact(path, receipt, zip_path):
    artifact = _load_json(path, "artifact")
    for field, value in (("id", receipt["artifact_id"]), ("name", receipt["artifact_name"])):
        if type(artifact.get(field)) is not type(value) or artifact[field] != value:
            raise ValueError(f"artifact {field} does not match receipt")
    if artifact.get("expired") is not False:
        raise ValueError("artifact is expired or has invalid expired flag")
    if artifact.get("digest") != "sha256:" + receipt["artifact_sha256"]:
        raise ValueError("artifact API digest does not match receipt")
    size = artifact.get("size_in_bytes")
    if not _positive_int(size) or size != Path(zip_path).stat().st_size:
        raise ValueError("artifact compressed size does not match ZIP")
    link = artifact.get("workflow_run")
    if not isinstance(link, dict) or type(link.get("id")) is not int or link["id"] != receipt["run_id"] or link.get("head_sha") != receipt["head_sha"]:
        raise ValueError("artifact workflow_run does not match receipt")


def _sha256(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _archive_name(name, directory):
    if (not isinstance(name, str) or not name or "\\" in name or name.startswith("/")
            or any(ord(character) < 32 or ord(character) == 127 for character in name)):
        raise ValueError(f"unsafe archive path: {name!r}")
    if unicodedata.normalize("NFC", name) != name:
        raise ValueError(f"noncanonical archive path: {name!r}")
    core = name[:-1] if directory else name
    parts = core.split("/")
    if (not core or any(part in ("", ".", "..") for part in parts)
            or re.match(r"[A-Za-z]:", parts[0])):
        raise ValueError(f"noncanonical archive path: {name!r}")
    return core


def _validate_archive(archive):
    files = set()
    seen = set()
    for member in archive.infolist():
        name = member.filename
        if member.orig_filename != name:
            raise ValueError(f"archive name contains NUL: {member.orig_filename!r}")
        directory = name.endswith("/")
        if bool(member.is_dir()) != directory or member.flag_bits & 1:
            raise ValueError(f"invalid or encrypted archive member: {name!r}")
        kind = stat.S_IFMT(member.external_attr >> 16) if member.create_system == 3 else 0
        if kind not in (0, stat.S_IFDIR if directory else stat.S_IFREG):
            raise ValueError(f"symlink or special archive member: {name!r}")
        core = _archive_name(name, directory)
        if core in seen:
            raise ValueError(f"duplicate or colliding archive path: {name!r}")
        seen.add(core)
        if not directory:
            files.add(core)
    for path in files:
        parts = path.split("/")
        if any("/".join(parts[:count]) in files for count in range(1, len(parts))):
            raise ValueError(f"file/directory collision: {path!r}")
    if f"{BUNDLE}/{MANIFEST}" not in files:
        raise ValueError("archive lacks shipped manifest")
    return files


def _extract(archive, output_dir):
    output_dir.mkdir()
    for member in archive.infolist():
        destination = output_dir / member.filename
        if member.is_dir():
            destination.mkdir(parents=True, exist_ok=True)
            continue
        destination.parent.mkdir(parents=True, exist_ok=True)
        with archive.open(member) as source, destination.open("xb") as target:
            for chunk in iter(lambda: source.read(1024 * 1024), b""):
                target.write(chunk)


def _manifest(bundle, expected_hash, archive_files):
    path = bundle / MANIFEST
    if _sha256(path) != expected_hash:
        raise ValueError("manifest digest does not match receipt")
    recorded = {}
    previous = ""
    try:
        with path.open("rb") as source:
            if source.readline(65537) != b"sha256\tpath\n":
                raise ValueError("invalid manifest header")
            while line := source.readline(65537):
                if len(line) > 65536 or not line.endswith(b"\n"):
                    raise ValueError("malformed manifest row")
                digest, separator, raw_name = line[:-1].partition(b"\t")
                if separator != b"\t" or not _matches(digest.decode("ascii"), HEX64):
                    raise ValueError("malformed manifest digest row")
                name = raw_name.decode("utf-8")
                _archive_name(f"{BUNDLE}/{name}", False)
                if "\t" in name or "\n" in name or name == MANIFEST or name <= previous:
                    raise ValueError(f"unsorted, duplicate, or invalid manifest path: {name!r}")
                recorded[name] = digest.decode("ascii")
                previous = name
    except UnicodeError as exc:
        raise ValueError(f"invalid UTF-8 manifest: {exc}") from exc
    if "oracle.json" not in recorded:
        raise ValueError("manifest lacks oracle.json")
    actual = {name[len(BUNDLE) + 1:] for name in archive_files if name.startswith(f"{BUNDLE}/")}
    if actual != set(recorded) | {MANIFEST}:
        raise ValueError("manifest file set differs from extracted bundle")
    for name, digest in recorded.items():
        if _sha256(bundle / name) != digest:
            raise ValueError(f"manifest file digest mismatch: {name}")


def verify_artifact(receipt_path, run_path, artifact_path, zip_path, output_dir):
    """Return the verified bundle root; never remove partial output on failure."""
    output_dir = Path(output_dir)
    if os.path.lexists(output_dir):
        raise ValueError(f"output directory already exists: {output_dir}")
    for ancestor in output_dir.parents:
        if ancestor.is_symlink():
            raise ValueError(f"output parent is a symlink: {ancestor}")
    receipt = _receipt(receipt_path)
    _run(run_path, receipt)
    _artifact(artifact_path, receipt, zip_path)
    if _sha256(zip_path) != receipt["artifact_sha256"]:
        raise ValueError("ZIP digest does not match receipt/API")
    try:
        with zipfile.ZipFile(zip_path) as archive:
            archive_files = _validate_archive(archive)
            bad = archive.testzip()
            if bad is not None:
                raise ValueError(f"corrupt ZIP member: {bad}")
            _extract(archive, output_dir)
    except (zipfile.BadZipFile, EOFError, zlib.error, RuntimeError, NotImplementedError) as exc:
        raise ValueError(f"invalid ZIP: {exc}") from exc
    bundle = output_dir / BUNDLE
    _manifest(bundle, receipt["manifest_sha256"], archive_files)
    return bundle


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--receipt", required=True, type=Path)
    parser.add_argument("--run-json", required=True, type=Path)
    parser.add_argument("--artifact-json", required=True, type=Path)
    parser.add_argument("--zip", required=True, type=Path)
    parser.add_argument("--output-dir", required=True, type=Path)
    args = parser.parse_args(argv)
    try:
        bundle = verify_artifact(args.receipt, args.run_json, args.artifact_json, args.zip, args.output_dir)
    except (ValueError, OSError) as exc:
        parser.exit(1, f"inferCNV artifact verification failed: {exc}\n")
    receipt = _receipt(args.receipt)
    print(f"verified bundle: {bundle}")
    print(f"run={receipt['run_id']} attempt={receipt['run_attempt']} head={receipt['head_sha']} artifact={receipt['artifact_id']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
