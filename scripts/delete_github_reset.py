#!/usr/bin/env python3
"""Delete registry-reset GitHub repositories only after validating their backups."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import time
from pathlib import Path


NAME_PATTERN = re.compile(r"^rsomics-[a-z0-9][a-z0-9-]*$")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--candidates",
        type=Path,
        default=Path(
            ".autopilot/state/registry-reset-github-delete-candidates-2026-07-30.txt"
        ),
    )
    parser.add_argument(
        "--keep",
        type=Path,
        default=Path("docs/00-overview/registry-reset-keep.txt"),
    )
    parser.add_argument(
        "--backup-root",
        type=Path,
        default=Path(
            "/Volumes/KIOXIA/Documents/omics-rust/_retired/"
            "registry-reset-2026-07-30/github"
        ),
    )
    parser.add_argument(
        "--progress",
        type=Path,
        default=Path(".autopilot/state/registry-reset-github-deleted-2026-07-30.txt"),
    )
    parser.add_argument(
        "--failures",
        type=Path,
        default=Path(
            ".autopilot/state/registry-reset-github-failures-2026-07-30.jsonl"
        ),
    )
    parser.add_argument("--organization", default="omics-rust")
    parser.add_argument("--delay", type=float, default=1.0)
    parser.add_argument("--limit", type=int)
    parser.add_argument("--preflight-only", action="store_true")
    return parser.parse_args()


def read_names(path: Path) -> list[str]:
    names = [
        line.strip()
        for line in path.read_text().splitlines()
        if line.strip() and not line.lstrip().startswith("#")
    ]
    if len(names) != len(set(names)):
        raise SystemExit(f"duplicate names in {path}")
    invalid = [name for name in names if not NAME_PATTERN.fullmatch(name)]
    if invalid:
        raise SystemExit(f"invalid repository names in {path}: {invalid}")
    return names


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def run(command: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(command, check=False, capture_output=True, text=True)


def validate_backup(
    name: str, organization: str, backup_root: Path
) -> dict[str, object]:
    repository = f"{organization}/{name}"
    crate_backup = (backup_root / name).resolve()
    manifest_path = crate_backup / "manifest.json"
    if not manifest_path.is_file():
        raise RuntimeError(f"missing manifest: {manifest_path}")

    manifest = json.loads(manifest_path.read_text())
    if manifest.get("repository") != repository:
        raise RuntimeError(
            f"manifest repository mismatch for {name}: {manifest.get('repository')}"
        )

    bundle = Path(str(manifest.get("bundle", ""))).resolve()
    if bundle.parent != crate_backup or not bundle.is_file():
        raise RuntimeError(f"bundle escapes backup directory or is missing: {bundle}")
    if bundle.stat().st_size != manifest.get("bundle_size"):
        raise RuntimeError(f"bundle size mismatch: {bundle}")
    if sha256(bundle) != manifest.get("bundle_sha256"):
        raise RuntimeError(f"bundle checksum mismatch: {bundle}")

    verified = run(["git", "bundle", "verify", str(bundle)])
    if verified.returncode != 0:
        raise RuntimeError(
            f"git bundle verification failed for {name}: {verified.stderr.strip()}"
        )
    return manifest


def append_jsonl(path: Path, payload: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a") as destination:
        destination.write(json.dumps(payload, sort_keys=True) + "\n")


def remote_inventory(organization: str) -> dict[str, int]:
    result = run(
        [
            "gh",
            "api",
            "--paginate",
            "--slurp",
            f"orgs/{organization}/repos?per_page=100",
        ]
    )
    if result.returncode != 0:
        raise RuntimeError(
            f"failed to inventory GitHub organization: {result.stderr.strip()}"
        )
    pages = json.loads(result.stdout)
    return {
        repository["name"]: int(repository["id"])
        for page in pages
        for repository in page
    }


def main() -> None:
    args = parse_args()
    candidates = read_names(args.candidates)
    keep = set(read_names(args.keep))
    protected = keep | {"rsomics-world", "rsomics-igzip"}
    overlap = sorted(set(candidates) & protected)
    if overlap:
        raise SystemExit(f"protected repositories appear in candidates: {overlap}")

    manifests = {
        name: validate_backup(name, args.organization, args.backup_root)
        for name in candidates
    }
    print(f"offline preflight passed for {len(manifests)} repositories")
    if args.preflight_only:
        return

    args.progress.parent.mkdir(parents=True, exist_ok=True)
    completed = set(read_names(args.progress)) if args.progress.exists() else set()
    unknown_completed = sorted(completed - set(candidates))
    if unknown_completed:
        raise SystemExit(f"progress contains non-candidates: {unknown_completed}")

    pending = [name for name in candidates if name not in completed]
    if args.limit is not None:
        pending = pending[: args.limit]

    inventory = remote_inventory(args.organization)
    unexpectedly_present = sorted(completed & inventory.keys())
    if unexpectedly_present:
        raise SystemExit(
            f"progress repositories still exist remotely: {unexpectedly_present}"
        )
    for name in candidates:
        if name in completed:
            continue
        current_id = inventory.get(name)
        if current_id is None:
            raise SystemExit(f"pending repository is absent before deletion: {name}")
        manifest_id = int(manifests[name]["github"]["id"])
        if current_id != manifest_id:
            raise SystemExit(
                f"remote identity mismatch for {name}: "
                f"backup={manifest_id}, current={current_id}"
            )

    deleted_this_run: list[str] = []
    for name in pending:
        repository = f"{args.organization}/{name}"
        deleted = run(["gh", "api", "--method", "DELETE", f"repos/{repository}"])
        if deleted.returncode != 0:
            append_jsonl(
                args.failures,
                {
                    "repository": repository,
                    "stage": "delete",
                    "error": deleted.stderr.strip(),
                },
            )
            raise SystemExit(
                f"delete failed for {repository}: {deleted.stderr.strip()}"
            )

        with args.progress.open("a") as destination:
            destination.write(name + "\n")
        completed.add(name)
        deleted_this_run.append(name)
        print(f"deleted {repository} ({len(completed)}/{len(candidates)})", flush=True)
        time.sleep(args.delay)

    remaining = remote_inventory(args.organization)
    still_present = sorted(set(deleted_this_run) & remaining.keys())
    if still_present:
        raise SystemExit(
            f"batch verification found repositories still present: {still_present}"
        )
    print(f"batch verification passed for {len(deleted_this_run)} repositories")


if __name__ == "__main__":
    main()
