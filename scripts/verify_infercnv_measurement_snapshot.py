#!/usr/bin/env python3
"""Verify a four-native-tested inferCNV source snapshot and extract exact bytes."""

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import tarfile
import tomllib
import unicodedata


TARGETS = ["linux-x86_64", "linux-aarch64", "macos-x86_64", "macos-aarch64"]
STEPS = {"Verify and extract frozen source", "Install Rust 1.91",
         "Verify dependency resolution and output path", "Test measurement support debug",
         "Test measurement support release and compile bench", "Test debug", "Test release",
         "Verify source remains unchanged", "Upload raw evidence"}
RECEIPT_KEYS = {"schema_version", "repository", "workflow_path", "run_id", "run_attempt",
                "head_sha", "targets", "snapshot_name", "archive_sha256", "manifest_sha256",
                "lock_sha256", "production_sha256"}
HASH = re.compile(r"[0-9a-f]{64}\Z")
SHA = re.compile(r"[0-9a-f]{40}\Z")
SNAPSHOT = re.compile(r"green-[1-9][0-9]*\Z")


def _unique_pairs(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def _read_json(path):
    try:
        return json.loads(Path(path).read_text(encoding="utf-8"), object_pairs_hook=_unique_pairs)
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        raise ValueError(f"cannot read JSON: {path}") from exc


def _hash_file(path):
    sha = hashlib.sha256()
    with Path(path).open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            sha.update(chunk)
    return sha.hexdigest()


def _regular(path):
    if not stat.S_ISREG(path.lstat().st_mode):
        raise ValueError(f"not a regular file: {path}")


def _no_symlink_path(path):
    for part in (path, *path.parents):
        if part.is_symlink():
            raise ValueError(f"symlinked path: {part}")


def _name(name):
    if not isinstance(name, str) or not name or name.startswith("/") or "\\" in name or ":" in name or \
            unicodedata.normalize("NFC", name) != name or any(ord(c) < 32 or ord(c) == 127 for c in name):
        raise ValueError(f"unsafe path: {name!r}")
    parts = name.split("/")
    if any(part in ("", ".", "..") for part in parts) or str(PurePosixPath(name)) != name:
        raise ValueError(f"unsafe path: {name!r}")
    return name


def _names_without_collisions(names):
    seen = set()
    for name in names:
        _name(name)
        if name in seen:
            raise ValueError(f"duplicate path: {name}")
        seen.add(name)
    for name in seen:
        parts = name.split("/")
        if any("/".join(parts[:i]) in seen for i in range(1, len(parts))):
            raise ValueError(f"file/directory collision: {name}")


def _manifest(data):
    try:
        text = data.decode("utf-8")
    except UnicodeError as exc:
        raise ValueError("manifest is not UTF-8") from exc
    if not text.endswith("\n") or "\r" in text:
        raise ValueError("manifest must be LF-terminated")
    rows = {}
    for line in text.splitlines():
        if len(line) < 67 or line[64:66] != "  " or not HASH.fullmatch(line[:64]):
            raise ValueError("malformed manifest row")
        name = _name(line[66:])
        if name in rows:
            raise ValueError("duplicate manifest path")
        rows[name] = line[:64]
    if not rows or list(rows) != sorted(rows):
        raise ValueError("manifest is empty or unsorted")
    _names_without_collisions(rows)
    return rows


def _receipt(data):
    if not isinstance(data, dict) or set(data) != RECEIPT_KEYS or type(data["schema_version"]) is not int or data["schema_version"] != 1:
        raise ValueError("receipt schema mismatch")
    if data["repository"] != "omics-rust/rsomics-world" or data["workflow_path"] != ".github/workflows/sc-cnv-core-candidate.yml":
        raise ValueError("receipt repository/workflow mismatch")
    if any(type(data[key]) is not int or data[key] <= 0 for key in ("run_id", "run_attempt")) or \
            not isinstance(data["head_sha"], str) or not SHA.fullmatch(data["head_sha"]):
        raise ValueError("receipt run identity mismatch")
    if data["targets"] != TARGETS or not isinstance(data["snapshot_name"], str) or \
            not SNAPSHOT.fullmatch(data["snapshot_name"]):
        raise ValueError("receipt target/snapshot mismatch")
    for key in ("archive_sha256", "manifest_sha256", "lock_sha256"):
        if not isinstance(data[key], str) or not HASH.fullmatch(data[key]):
            raise ValueError(f"invalid receipt hash: {key}")
    production = data["production_sha256"]
    if not isinstance(production, dict) or not production:
        raise ValueError("missing production baseline")
    _names_without_collisions(production)
    for path, value in production.items():
        if not path.startswith("src/") or not isinstance(value, str) or not HASH.fullmatch(value):
            raise ValueError("invalid production baseline")


def _run_jobs(receipt, run, jobs):
    if not isinstance(run, dict):
        raise ValueError("invalid run record")
    expected = {"id": receipt["run_id"], "run_attempt": receipt["run_attempt"],
                "head_sha": receipt["head_sha"], "path": receipt["workflow_path"],
                "event": "workflow_dispatch", "status": "completed", "conclusion": "success"}
    if any(run.get(key) != value or isinstance(run.get(key), bool) for key, value in expected.items()) or \
            run.get("repository", {}).get("full_name") != receipt["repository"] or \
            run.get("head_repository", {}).get("full_name") != receipt["repository"]:
        raise ValueError("native run identity/status mismatch")
    if not isinstance(jobs, dict) or type(jobs.get("total_count")) is not int or jobs["total_count"] != 4 or \
            not isinstance(jobs.get("jobs"), list) or len(jobs["jobs"]) != 4:
        raise ValueError("native job count mismatch")
    if {job.get("name") for job in jobs["jobs"]} != set(TARGETS):
        raise ValueError("native targets mismatch")
    for job in jobs["jobs"]:
        for key, value in (("run_id", receipt["run_id"]), ("run_attempt", receipt["run_attempt"]),
                           ("head_sha", receipt["head_sha"]), ("status", "completed"),
                           ("conclusion", "success")):
            if job.get(key) != value or isinstance(job.get(key), bool):
                raise ValueError(f"native job {job.get('name')} identity/status mismatch")
        steps = job.get("steps")
        if not isinstance(steps, list):
            raise ValueError("missing native steps")
        required = STEPS | ({"Format and strict Clippy"} if job["name"] == "linux-x86_64" else set())
        required_records = [step.get("name") for step in steps if step.get("name") in required]
        if len(required_records) != len(set(required_records)):
            raise ValueError(f"native job {job['name']} duplicate required step")
        successful = [step.get("name") for step in steps if step.get("status") == "completed" and step.get("conclusion") == "success"]
        if len(successful) != len(set(successful)) or not required.issubset(successful):
            raise ValueError(f"native job {job['name']} missing/duplicate/failed required step")
        external = [step for step in steps if step.get("name") == "Acquire and verify pinned external oracle"]
        if len(external) != 1 or external[0].get("status") != "completed" or \
                external[0].get("conclusion") not in ("success", "skipped"):
            raise ValueError("external oracle step inconsistent")
    external_status = {next(step["conclusion"] for step in job["steps"] if step.get("name") == "Acquire and verify pinned external oracle") for job in jobs["jobs"]}
    if len(external_status) != 1:
        raise ValueError("mixed external oracle selection")
    linux = next(job for job in jobs["jobs"] if job["name"] == "linux-x86_64")
    gate = [step for step in linux["steps"] if step.get("name") == "Require explicit external oracle configuration"]
    if len(gate) != 1 or gate[0].get("status") != "completed" or \
            gate[0].get("conclusion") != ("success" if "success" in external_status else "skipped"):
        raise ValueError("external oracle configuration mismatch")


def _check_cargo(root):
    try:
        cargo = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    except (OSError, UnicodeError, tomllib.TOMLDecodeError) as exc:
        raise ValueError("invalid Cargo.toml") from exc
    bench = [b for b in cargo.get("bench", []) if b.get("name") == "cnv_matched"]
    nix = cargo.get("target", {}).get('cfg(target_os = "linux")', {}).get("dev-dependencies", {}).get("nix", {})
    if cargo.get("package", {}).get("name") != "rsomics-sc" or cargo.get("package", {}).get("rust-version") != "1.91" or \
            cargo.get("dependencies", {}).get("rsomics-common") != "=0.12.3" or \
            cargo.get("features", {}).get("infercnv-measurement") != [] or \
            len(bench) != 1 or bench[0].get("harness") is not False or bench[0].get("test") is not False or \
            bench[0].get("required-features") != ["infercnv-measurement"] or \
            nix != {"version": "=0.29.0", "default-features": False, "features": ["resource"]}:
        raise ValueError("Cargo measurement-only configuration mismatch")


def verify_extracted(receipt_path, snapshot_dir, source_root):
    receipt_path, snapshot_dir, source_root = map(Path, (receipt_path, snapshot_dir, source_root))
    receipt = _read_json(receipt_path)
    _receipt(receipt)
    expected_snapshot = receipt_path.parent.parent / "snapshots/sc-cnv-core-2026-09-26" / receipt["snapshot_name"]
    if snapshot_dir.absolute() != expected_snapshot.absolute() or not snapshot_dir.is_dir() or \
            not source_root.is_dir():
        raise ValueError("snapshot or source path mismatch")
    _no_symlink_path(snapshot_dir)
    _no_symlink_path(source_root)
    if _hash_file(snapshot_dir / "source.tar.gz") != receipt["archive_sha256"] or \
            _hash_file(snapshot_dir / "files.sha256") != receipt["manifest_sha256"]:
        raise ValueError("snapshot changed")
    manifest = _manifest((snapshot_dir / "files.sha256").read_bytes())
    present = {}
    for path in source_root.rglob("*"):
        if path.is_symlink():
            raise ValueError(f"source link appeared: {path}")
        if path.is_dir():
            continue
        _regular(path)
        present[path.relative_to(source_root).as_posix()] = _hash_file(path)
    if present != manifest or present.get("Cargo.lock") != receipt["lock_sha256"] or \
            {name: value for name, value in present.items() if name.startswith("src/")} != receipt["production_sha256"]:
        raise ValueError("source/lock bytes or file set changed")
    _check_cargo(source_root)
    return {"archive_sha256": receipt["archive_sha256"], "manifest_sha256": receipt["manifest_sha256"],
            "lock_sha256": receipt["lock_sha256"], "verified_files": len(present)}


def record_extracted_check(receipt_path, snapshot_dir, source_root, evidence_dir):
    evidence_dir = Path(evidence_dir)
    try:
        result = verify_extracted(receipt_path, snapshot_dir, source_root)
        (evidence_dir / "source-after.json").write_text(json.dumps(result, sort_keys=True) + "\n", encoding="utf-8")
    except (OSError, ValueError) as exc:
        (evidence_dir / "source-after.status").write_text("failed\n", encoding="utf-8")
        (evidence_dir / "source-after.error.txt").write_text(f"{type(exc).__name__}: {exc}\n", encoding="utf-8")
        raise
    (evidence_dir / "source-after.status").write_text("passed\n", encoding="utf-8")
    return result


def verify_snapshot(receipt_path, run_path, jobs_path, snapshot_dir, output_dir):
    receipt_path, snapshot_dir, output_dir = map(Path, (receipt_path, snapshot_dir, output_dir))
    receipt = _read_json(receipt_path)
    _receipt(receipt)
    _run_jobs(receipt, _read_json(run_path), _read_json(jobs_path))
    expected_snapshot = receipt_path.parent.parent / "snapshots/sc-cnv-core-2026-09-26" / receipt["snapshot_name"]
    if snapshot_dir.absolute() != expected_snapshot.absolute() or not snapshot_dir.is_dir():
        raise ValueError("snapshot directory differs from trusted selector")
    _no_symlink_path(snapshot_dir)
    _no_symlink_path(output_dir.parent)
    if output_dir.exists() or output_dir.is_symlink():
        raise ValueError("output directory already exists")
    for name in ("README.md", "checksums.sha256", "files.sha256", "source.tar.gz"):
        _regular(snapshot_dir / name)
    if _hash_file(snapshot_dir / "source.tar.gz") != receipt["archive_sha256"] or \
            _hash_file(snapshot_dir / "files.sha256") != receipt["manifest_sha256"]:
        raise ValueError("snapshot digest mismatch")
    checks = _manifest((snapshot_dir / "checksums.sha256").read_bytes())
    if set(checks) != {"README.md", "files.sha256", "source.tar.gz"} or \
            any(_hash_file(snapshot_dir / name) != value for name, value in checks.items()):
        raise ValueError("snapshot checksums mismatch")
    manifest = _manifest((snapshot_dir / "files.sha256").read_bytes())
    if "files.sha256" in manifest or "Cargo.lock" not in manifest or "Cargo.toml" not in manifest or \
            manifest["Cargo.lock"] != receipt["lock_sha256"]:
        raise ValueError("source manifest/lock mismatch")
    source = {name: value for name, value in manifest.items() if name.startswith("src/")}
    if source != receipt["production_sha256"]:
        raise ValueError("production source manifest mismatch")
    with tarfile.open(snapshot_dir / "source.tar.gz", "r:gz") as archive:
        members = archive.getmembers()
        names = [member.name for member in members]
        _names_without_collisions(names)
        if set(names) != set(manifest) or any(not member.isfile() or member.type != tarfile.REGTYPE for member in members):
            raise ValueError("archive names/types differ from source manifest")
        output_dir.mkdir(parents=True)
        for member in members:
            destination = output_dir.joinpath(*member.name.split("/"))
            destination.parent.mkdir(parents=True, exist_ok=True)
            sha = hashlib.sha256()
            with archive.extractfile(member) as source_file, destination.open("xb") as target:
                for chunk in iter(lambda: source_file.read(1024 * 1024), b""):
                    target.write(chunk)
                    sha.update(chunk)
            if sha.hexdigest() != manifest[member.name]:
                raise ValueError(f"extracted hash mismatch: {member.name}")
    verify_extracted(receipt_path, snapshot_dir, output_dir)
    return output_dir


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("receipt", "run-json", "jobs-json", "snapshot-dir", "output-dir"):
        parser.add_argument("--" + name, required=True, type=Path)
    args = parser.parse_args()
    try:
        verify_snapshot(args.receipt, args.run_json, args.jobs_json, args.snapshot_dir, args.output_dir)
    except (OSError, ValueError, tarfile.TarError) as exc:
        parser.exit(1, f"snapshot verification failed: {exc}\n")


if __name__ == "__main__":
    main()
