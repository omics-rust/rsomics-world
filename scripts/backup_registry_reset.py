#!/usr/bin/env python3
"""Back up crates.io packages selected for the rsomics registry reset."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import tarfile
import time
import urllib.error
import urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from urllib.parse import quote


DEFAULT_CANDIDATES = Path(
    ".autopilot/state/registry-reset-cratesio-candidates-2026-07-30.txt"
)
DEFAULT_ROOT = Path(
    "/Volumes/KIOXIA/Documents/omics-rust/_retired/registry-reset-2026-07-30"
)
USER_AGENT = "rsomics-world-registry-reset-backup (contact: Bengerthelorf)"


def fetch(url: str, attempts: int = 6) -> bytes:
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    for attempt in range(attempts):
        try:
            with urllib.request.urlopen(request, timeout=60) as response:
                return response.read()
        except (OSError, urllib.error.HTTPError) as error:
            if attempt + 1 == attempts:
                raise RuntimeError(f"failed to fetch {url}: {error}") from error
            time.sleep(min(2**attempt, 8))
    raise AssertionError("retry loop exhausted")


def checksum(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def verify_archive(path: Path, expected: str) -> None:
    actual = checksum(path)
    if actual != expected:
        raise RuntimeError(f"{path}: checksum {actual} != {expected}")
    with tarfile.open(path, mode="r:gz") as archive:
        archive.getmembers()


def sparse_index_url(name: str) -> str:
    if len(name) == 1:
        return f"https://index.crates.io/1/{name}"
    if len(name) == 2:
        return f"https://index.crates.io/2/{name}"
    if len(name) == 3:
        return f"https://index.crates.io/3/{name[0]}/{name}"
    return f"https://index.crates.io/{name[:2]}/{name[2:4]}/{name}"


def backup_crate(name: str, root: Path) -> dict:
    crate_dir = root / "cratesio" / name
    manifest_path = root / "manifests" / "cratesio" / f"{name}.json"
    crate_dir.mkdir(parents=True, exist_ok=True)
    manifest_path.parent.mkdir(parents=True, exist_ok=True)

    index_bytes = fetch(sparse_index_url(name))
    entries = [
        json.loads(line)
        for line in index_bytes.decode("utf-8").splitlines()
        if line.strip()
    ]
    if not entries:
        raise RuntimeError(f"{name}: sparse index has no versions")

    versions = []
    for entry in entries:
        version = entry["vers"]
        expected = entry["cksum"]
        filename = f"{name}-{version}.crate"
        destination = crate_dir / filename
        if destination.exists():
            verify_archive(destination, expected)
        else:
            encoded_name = quote(name, safe="")
            encoded_filename = quote(filename, safe="")
            url = f"https://static.crates.io/crates/{encoded_name}/{encoded_filename}"
            payload = fetch(url)
            temporary = destination.with_name(f"{destination.name}.part")
            temporary.write_bytes(payload)
            verify_archive(temporary, expected)
            os.replace(temporary, destination)

        versions.append(
            {
                "version": version,
                "checksum": expected,
                "yanked": entry.get("yanked", False),
                "size": destination.stat().st_size,
                "path": str(destination),
            }
        )

    record = {
        "crate": name,
        "index_url": sparse_index_url(name),
        "versions": versions,
    }
    temporary_manifest = manifest_path.with_suffix(".json.part")
    temporary_manifest.write_text(
        json.dumps(record, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    os.replace(temporary_manifest, manifest_path)
    return record


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--candidates", type=Path, default=DEFAULT_CANDIDATES)
    parser.add_argument("--root", type=Path, default=DEFAULT_ROOT)
    parser.add_argument("--workers", type=int, default=8)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    root = args.root.resolve()
    if not str(root).startswith("/Volumes/KIOXIA/"):
        raise SystemExit(f"backup root must be on KIOXIA: {root}")

    names = [
        line.strip()
        for line in args.candidates.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.startswith("#")
    ]
    failures: list[tuple[str, str]] = []
    completed = 0
    version_count = 0
    byte_count = 0

    with ThreadPoolExecutor(max_workers=args.workers) as executor:
        futures = {executor.submit(backup_crate, name, root): name for name in names}
        for future in as_completed(futures):
            name = futures[future]
            try:
                record = future.result()
            except Exception as error:
                failures.append((name, str(error)))
            else:
                completed += 1
                version_count += len(record["versions"])
                byte_count += sum(item["size"] for item in record["versions"])
                if completed % 25 == 0 or completed == len(names):
                    print(
                        f"backed up {completed}/{len(names)} crates "
                        f"({version_count} versions, {byte_count} bytes)",
                        flush=True,
                    )

    summary = {
        "candidate_count": len(names),
        "completed_count": completed,
        "version_count": version_count,
        "byte_count": byte_count,
        "failures": [{"crate": name, "error": error} for name, error in failures],
    }
    summary_path = root / "manifests" / "cratesio-summary.json"
    summary_path.parent.mkdir(parents=True, exist_ok=True)
    summary_path.write_text(
        json.dumps(summary, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(json.dumps(summary, sort_keys=True))
    if failures:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
