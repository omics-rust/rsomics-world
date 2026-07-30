#!/usr/bin/env python3
"""Back up GitHub repositories selected for the rsomics registry reset."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import tarfile
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path


DEFAULT_CANDIDATES = Path(
    ".autopilot/state/registry-reset-github-candidates-2026-07-30.txt"
)
DEFAULT_CLONES = Path("/Volumes/KIOXIA/Documents/omics-rust")
DEFAULT_ROOT = Path(
    "/Volumes/KIOXIA/Documents/omics-rust/_retired/registry-reset-2026-07-30"
)
ORGANIZATION = "omics-rust"


def run(
    command: list[str],
    *,
    cwd: Path | None = None,
    text: bool = True,
) -> subprocess.CompletedProcess:
    return subprocess.run(
        command,
        cwd=cwd,
        check=True,
        capture_output=True,
        text=text,
    )


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def github_metadata(name: str) -> dict:
    result = run(["gh", "api", f"repos/{ORGANIZATION}/{name}"])
    return json.loads(result.stdout)


def save_dirty_state(repo: Path, output_dir: Path) -> dict:
    status = run(
        ["git", "status", "--porcelain=v1", "-z", "--untracked-files=all"],
        cwd=repo,
        text=False,
    ).stdout
    dirty = bool(status)
    record = {"dirty": dirty, "status_entries": []}
    if not dirty:
        return record

    status_path = output_dir / "status-porcelain-v1-z"
    status_path.write_bytes(status)
    record["status_path"] = str(status_path)
    record["status_entries"] = [
        item.decode("utf-8", errors="replace") for item in status.split(b"\0") if item
    ]

    diff = run(
        ["git", "diff", "--binary", "--no-ext-diff", "HEAD"],
        cwd=repo,
        text=False,
    ).stdout
    diff_path = output_dir / "working-tree.patch"
    diff_path.write_bytes(diff)
    record["diff_path"] = str(diff_path)

    untracked = run(
        ["git", "ls-files", "-z", "--others", "--exclude-standard"],
        cwd=repo,
        text=False,
    ).stdout
    untracked_paths = [
        Path(item.decode("utf-8", errors="surrogateescape"))
        for item in untracked.split(b"\0")
        if item
    ]
    if untracked_paths:
        archive_path = output_dir / "untracked.tar.gz"
        with tarfile.open(archive_path, mode="w:gz") as archive:
            for relative in untracked_paths:
                source = repo / relative
                if source.exists() or source.is_symlink():
                    archive.add(source, arcname=str(relative), recursive=True)
        record["untracked_archive"] = str(archive_path)
        record["untracked_sha256"] = sha256(archive_path)
        record["untracked_paths"] = [str(path) for path in untracked_paths]

    return record


def prepare_repository(name: str, clones: Path, root: Path) -> tuple[Path, bool]:
    local = clones / name
    if (local / ".git").is_dir():
        run(["git", "fetch", "--prune", "--tags", "origin"], cwd=local)
        return local, False

    mirror = root / "github-mirrors" / f"{name}.git"
    mirror.parent.mkdir(parents=True, exist_ok=True)
    if mirror.is_dir():
        run(["git", "fetch", "--prune", "--tags", "origin"], cwd=mirror)
    else:
        run(
            [
                "git",
                "clone",
                "--mirror",
                f"https://github.com/{ORGANIZATION}/{name}.git",
                str(mirror),
            ]
        )
    return mirror, True


def backup_repository(name: str, clones: Path, root: Path) -> dict:
    output_dir = root / "github" / name
    output_dir.mkdir(parents=True, exist_ok=True)
    repo, is_mirror = prepare_repository(name, clones, root)
    run(["git", "fsck", "--full"], cwd=repo)

    dirty_record = (
        {"dirty": False, "status_entries": []}
        if is_mirror
        else save_dirty_state(repo, output_dir)
    )

    bundle = output_dir / f"{name}.bundle"
    temporary_bundle = bundle.with_suffix(".bundle.part")
    if temporary_bundle.exists():
        temporary_bundle.unlink()
    run(["git", "bundle", "create", str(temporary_bundle), "--all"], cwd=repo)
    run(["git", "bundle", "verify", str(temporary_bundle)], cwd=repo)
    os.replace(temporary_bundle, bundle)

    refs = run(
        ["git", "for-each-ref", "--format=%(refname)\t%(objectname)"],
        cwd=repo,
    ).stdout.splitlines()
    head = run(["git", "rev-parse", "HEAD"], cwd=repo).stdout.strip()
    metadata = github_metadata(name)

    record = {
        "repository": f"{ORGANIZATION}/{name}",
        "source": str(repo),
        "source_is_mirror": is_mirror,
        "head": head,
        "refs": refs,
        "bundle": str(bundle),
        "bundle_sha256": sha256(bundle),
        "bundle_size": bundle.stat().st_size,
        "github": {
            "id": metadata["id"],
            "node_id": metadata["node_id"],
            "html_url": metadata["html_url"],
            "description": metadata["description"],
            "archived": metadata["archived"],
            "visibility": metadata["visibility"],
            "default_branch": metadata["default_branch"],
            "created_at": metadata["created_at"],
            "updated_at": metadata["updated_at"],
            "pushed_at": metadata["pushed_at"],
            "open_issues_count": metadata["open_issues_count"],
            "forks_count": metadata["forks_count"],
            "stargazers_count": metadata["stargazers_count"],
            "watchers_count": metadata["watchers_count"],
        },
        "working_tree": dirty_record,
    }
    manifest = output_dir / "manifest.json"
    temporary_manifest = manifest.with_suffix(".json.part")
    temporary_manifest.write_text(
        json.dumps(record, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    os.replace(temporary_manifest, manifest)
    return record


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--candidates", type=Path, default=DEFAULT_CANDIDATES)
    parser.add_argument("--clones", type=Path, default=DEFAULT_CLONES)
    parser.add_argument("--root", type=Path, default=DEFAULT_ROOT)
    parser.add_argument("--workers", type=int, default=6)
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
    completed = 0
    dirty = 0
    byte_count = 0
    failures: list[tuple[str, str]] = []

    with ThreadPoolExecutor(max_workers=args.workers) as executor:
        futures = {
            executor.submit(backup_repository, name, args.clones, root): name
            for name in names
        }
        for future in as_completed(futures):
            name = futures[future]
            try:
                record = future.result()
            except Exception as error:
                failures.append((name, str(error)))
            else:
                completed += 1
                dirty += int(record["working_tree"]["dirty"])
                byte_count += record["bundle_size"]
                if completed % 25 == 0 or completed == len(names):
                    print(
                        f"backed up {completed}/{len(names)} repositories "
                        f"({dirty} dirty, {byte_count} bundle bytes)",
                        flush=True,
                    )

    summary = {
        "candidate_count": len(names),
        "completed_count": completed,
        "dirty_count": dirty,
        "bundle_bytes": byte_count,
        "failures": [{"repository": name, "error": error} for name, error in failures],
    }
    summary_path = root / "manifests" / "github-summary.json"
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
