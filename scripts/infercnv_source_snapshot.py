"""Private checksum, logical tar-member and exact-byte source snapshot guard."""

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import stat
import tarfile
import unicodedata


HASH = re.compile(r"[0-9a-f]{64}\Z")
CHECKSUM_FILES = {"README.md", "files.sha256", "source.tar.gz"}


def _hash_file(path):
    with Path(path).open("rb") as source:
        return _hash_stream(source)


def _hash_stream(source):
    sha = hashlib.sha256()
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


def read_snapshot_manifest(snapshot_dir):
    snapshot_dir = Path(snapshot_dir)
    _no_symlink_path(snapshot_dir)
    if not snapshot_dir.is_dir():
        raise ValueError("snapshot directory absent")
    for name in CHECKSUM_FILES | {"checksums.sha256"}:
        _regular(snapshot_dir / name)
    checks = _manifest((snapshot_dir / "checksums.sha256").read_bytes())
    if set(checks) != CHECKSUM_FILES or any(_hash_file(snapshot_dir / name) != value for name, value in checks.items()):
        raise ValueError("snapshot checksums mismatch")
    return _manifest((snapshot_dir / "files.sha256").read_bytes())


def _verified_members(archive, manifest):
    members = archive.getmembers()
    names = [member.name for member in members]
    _names_without_collisions(names)
    if set(names) != set(manifest) or any(not member.isfile() or member.type != tarfile.REGTYPE for member in members):
        raise ValueError("archive names/types differ from source manifest")
    for member in members:
        with archive.extractfile(member) as source:
            if _hash_stream(source) != manifest[member.name]:
                raise ValueError(f"archive hash mismatch: {member.name}")
    return members


def verify_and_extract(snapshot_dir, output_dir, manifest=None):
    snapshot_dir, output_dir = map(Path, (snapshot_dir, output_dir))
    actual_manifest = read_snapshot_manifest(snapshot_dir)
    if manifest is not None and actual_manifest != manifest:
        raise ValueError("source manifest changed")
    manifest = actual_manifest
    _no_symlink_path(output_dir.parent)
    if output_dir.exists() or output_dir.is_symlink():
        raise ValueError("output directory already exists")
    with tarfile.open(snapshot_dir / "source.tar.gz", "r:gz") as archive:
        members = _verified_members(archive, manifest)
        output_dir.mkdir(parents=True)
        for member in members:
            destination = output_dir.joinpath(*member.name.split("/"))
            destination.parent.mkdir(parents=True, exist_ok=True)
            sha = hashlib.sha256()
            with archive.extractfile(member) as source, destination.open("xb") as target:
                for chunk in iter(lambda: source.read(1024 * 1024), b""):
                    target.write(chunk)
                    sha.update(chunk)
            if sha.hexdigest() != manifest[member.name]:
                raise ValueError(f"extracted hash mismatch: {member.name}")
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--snapshot-dir", required=True, type=Path)
    parser.add_argument("--output-dir", required=True, type=Path)
    args = parser.parse_args()
    try:
        manifest = verify_and_extract(args.snapshot_dir, args.output_dir)
        print(json.dumps({"verified_files": len(manifest),
                          "archive_sha256": _hash_file(args.snapshot_dir / "source.tar.gz"),
                          "manifest_sha256": _hash_file(args.snapshot_dir / "files.sha256")}, sort_keys=True))
    except (OSError, ValueError, tarfile.TarError) as exc:
        parser.exit(1, f"source snapshot verification failed: {exc}\n")


if __name__ == "__main__":
    main()
