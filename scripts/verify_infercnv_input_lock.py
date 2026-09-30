"""Verify that raw-input dependency resolution preserves every existing node."""

import argparse
import re
import tomllib
from pathlib import Path


ADDED_NAMES = {"flate2", "crc32fast", "miniz_oxide", "adler2", "simd-adler32"}


def _packages(document):
    result = {}
    for package in document["package"]:
        key = (package["name"], package["version"], package.get("source"))
        if key in result:
            raise ValueError("duplicate lock package")
        result[key] = package
    return result


def verify_lock(before: dict, after: dict) -> None:
    if before.keys() != after.keys() or before["version"] != after["version"]:
        raise ValueError("lock schema changed")
    old, new = _packages(before), _packages(after)
    if not old.keys() <= new.keys():
        raise ValueError("existing lock package removed or changed")
    root_count = 0
    for key, package in old.items():
        expected = dict(package)
        if key[0] == "rsomics-sc":
            root_count += 1
            expected["dependencies"] = sorted(package["dependencies"] + ["flate2"])
            if new[key] != expected:
                raise ValueError("unexpected root dependency change")
        elif new[key] != package:
            raise ValueError(f"existing package changed: {key}")
    if root_count != 1:
        raise ValueError("expected one root package")
    added = [new[key] for key in new.keys() - old.keys()]
    if len(added) != len(ADDED_NAMES) or {p["name"] for p in added} != ADDED_NAMES:
        raise ValueError("unexpected added dependency nodes")
    for package in added:
        if package.get("source") != "registry+https://github.com/rust-lang/crates.io-index" or \
                re.fullmatch(r"[0-9a-f]{64}", package.get("checksum", "")) is None:
            raise ValueError("invalid registry dependency identity")
        if package["name"] == "flate2" and package["version"] != "1.1.9":
            raise ValueError("incorrect flate2 version")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("before", type=Path)
    parser.add_argument("after", type=Path)
    args = parser.parse_args()
    verify_lock(tomllib.loads(args.before.read_text()), tomllib.loads(args.after.read_text()))
    print("Existing lock packages preserved; only raw-input gzip nodes added.")
