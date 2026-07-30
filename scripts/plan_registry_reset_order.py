#!/usr/bin/env python3
"""Plan a crates.io deletion order from the live sparse-index dependency graph."""

from __future__ import annotations

import argparse
import json
import time
import urllib.error
import urllib.request
from collections import defaultdict, deque
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from urllib.parse import urlencode


DEFAULT_KEEP = Path("docs/00-overview/registry-reset-keep.txt")
DEFAULT_STATE = Path(".autopilot/state")
USER_AGENT = "rsomics-world-registry-reset-planner (contact: Bengerthelorf)"


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


def sparse_index_url(name: str) -> str:
    if len(name) == 1:
        return f"https://index.crates.io/1/{name}"
    if len(name) == 2:
        return f"https://index.crates.io/2/{name}"
    if len(name) == 3:
        return f"https://index.crates.io/3/{name[0]}/{name}"
    return f"https://index.crates.io/{name[:2]}/{name[2:4]}/{name}"


def live_rsomics_crates() -> set[str]:
    names: set[str] = set()
    page = 1
    while True:
        query = urlencode({"q": "rsomics-", "per_page": 100, "page": page})
        payload = json.loads(fetch(f"https://crates.io/api/v1/crates?{query}"))
        page_names = {crate["id"] for crate in payload["crates"]}
        names.update(page_names)
        if len(names) >= payload["meta"]["total"] or not page_names:
            return names
        page += 1


def dependencies_for(name: str) -> set[str]:
    dependencies: set[str] = set()
    content = fetch(sparse_index_url(name)).decode("utf-8")
    for line in content.splitlines():
        if not line.strip():
            continue
        version = json.loads(line)
        for dependency in version.get("deps", []):
            dependency_name = dependency.get("package") or dependency["name"]
            dependencies.add(dependency_name.replace("_", "-"))
    dependencies.discard(name)
    return dependencies


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--keep", type=Path, default=DEFAULT_KEEP)
    parser.add_argument("--state", type=Path, default=DEFAULT_STATE)
    parser.add_argument("--workers", type=int, default=12)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    keep_policy = {
        line.strip()
        for line in args.keep.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.startswith("#")
    }
    live = live_rsomics_crates()
    keep = live & keep_policy
    candidates = live - keep_policy

    dependency_map: dict[str, set[str]] = {}
    failures: list[tuple[str, str]] = []
    with ThreadPoolExecutor(max_workers=args.workers) as executor:
        futures = {
            executor.submit(dependencies_for, name): name for name in sorted(live)
        }
        for future in as_completed(futures):
            name = futures[future]
            try:
                dependency_map[name] = future.result() & live
            except Exception as error:
                failures.append((name, str(error)))

    if failures:
        for name, error in failures:
            print(f"{name}: {error}")
        raise SystemExit(1)

    reverse: dict[str, set[str]] = defaultdict(set)
    for dependent, dependencies in dependency_map.items():
        for dependency in dependencies:
            reverse[dependency].add(dependent)

    blockers = {
        candidate: sorted(reverse[candidate] & keep)
        for candidate in candidates
        if reverse[candidate] & keep
    }
    deletable = candidates - blockers.keys()

    outgoing = {name: dependency_map[name] & deletable for name in deletable}
    indegree = {name: 0 for name in deletable}
    for dependencies in outgoing.values():
        for dependency in dependencies:
            indegree[dependency] += 1

    queue = deque(sorted(name for name, degree in indegree.items() if degree == 0))
    order: list[str] = []
    while queue:
        name = queue.popleft()
        order.append(name)
        for dependency in sorted(outgoing[name]):
            indegree[dependency] -= 1
            if indegree[dependency] == 0:
                queue.append(dependency)

    cycles = sorted(deletable - set(order))
    state = args.state
    state.mkdir(parents=True, exist_ok=True)
    (state / "registry-reset-delete-order-2026-07-30.txt").write_text(
        "\n".join(order) + ("\n" if order else ""),
        encoding="utf-8",
    )
    report = {
        "live_count": len(live),
        "keep_policy_count": len(keep_policy),
        "live_keep_count": len(keep),
        "candidate_count": len(candidates),
        "deletable_count": len(order),
        "blockers": blockers,
        "cycles": cycles,
        "dependency_edges": sum(len(value) for value in dependency_map.values()),
    }
    (state / "registry-reset-delete-order-2026-07-30.json").write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(json.dumps(report, indent=2, sort_keys=True))
    if cycles:
        raise SystemExit(2)


if __name__ == "__main__":
    main()
