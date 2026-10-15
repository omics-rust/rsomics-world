"""Capture the actual Python process identity for private paired trials."""

import hashlib
import json
from pathlib import Path
import re
import stat
import sys


def _parse_maps(text: str) -> list[str]:
    rows = text.splitlines()
    if not rows:
        raise ValueError("empty /proc/self/maps")
    libraries = set()
    for number, row in enumerate(rows, 1):
        columns = row.split(None, 5)
        if len(columns) < 5:
            raise ValueError(f"malformed /proc/self/maps row {number}")
        address, permissions, offset, device, inode = columns[:5]
        if not (
            re.fullmatch(r"[0-9a-fA-F]+-[0-9a-fA-F]+", address)
            and re.fullmatch(r"[r-][w-][x-][ps]", permissions)
            and re.fullmatch(r"[0-9a-fA-F]+", offset)
            and re.fullmatch(r"[0-9a-fA-F]+:[0-9a-fA-F]+", device)
            and re.fullmatch(r"[0-9]+", inode)
        ):
            raise ValueError(f"malformed /proc/self/maps row {number}")
        start, end = (int(value, 16) for value in address.split("-"))
        if start >= end or "\x00" in row:
            raise ValueError(f"malformed /proc/self/maps row {number}")
        if len(columns) == 6:
            pathname = columns[5]
            path = Path(pathname)
            if path.name.startswith("libpython"):
                if pathname.endswith(" (deleted)"):
                    raise ValueError(f"deleted runtime mapping on row {number}")
                if not path.is_absolute():
                    raise ValueError(f"nonabsolute runtime mapping on row {number}")
                libraries.add(pathname)
    return sorted(libraries)


def _hash_file(path: Path) -> str:
    if not stat.S_ISREG(path.stat().st_mode):
        raise OSError(f"not a regular runtime file: {path}")
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def capture_identity() -> dict:
    """Report actual runtime files and interpreter fields; require live procfs."""
    libraries = _parse_maps(Path("/proc/self/maps").read_text(encoding="utf-8"))
    executable = Path(sys.executable).resolve(strict=True)
    return {
        "executable": str(executable),
        "executable_sha256": _hash_file(executable),
        "version": sys.version,
        "prefix": sys.prefix,
        "base_prefix": sys.base_prefix,
        "stdlib_json": json.__file__,
        "libpython_sha256": {path: _hash_file(Path(path)) for path in libraries},
    }


def main() -> int:
    if len(sys.argv) != 1:
        print("infercnv Python identity: no arguments accepted", file=sys.stderr)
        return 2
    try:
        identity = capture_identity()
    except (OSError, ValueError) as error:
        diagnostic = " ".join(str(error).splitlines())
        print(f"infercnv Python identity: {diagnostic}", file=sys.stderr)
        return 1
    print(json.dumps(identity, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
