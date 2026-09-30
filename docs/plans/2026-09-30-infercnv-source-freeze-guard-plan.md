# Shared frozen-source guard implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans. Root integrates and observes RED/GREEN; bounded reviewers are read-only.

**Goal:** Reject undeclared archive source members and incomplete snapshot checksums before any candidate compilation, while sharing the existing receipt verifier's source rules.

**Architecture:** Move existing private hash/path/manifest rules into one controller module used by both source-candidate and four-native receipt gates. Validate all logical source members and hashes before creating output, preserve legal PAX representation, and retain existing receipt/run/Cargo/post-trial policies. A clean green-12 packages exactly the same Rust bytes as green-11.

**Tech Stack:** Python standard library (>=3.11), tarfile, SHA256; no build, installation or download for controller tests.

**Spec:** The strict source archive contract already implemented by scripts/verify_infercnv_measurement_snapshot.py plus docs/plans/2026-09-30-infercnv-private-creation-support-design.md. This repairs candidate packaging parity, not numerical behavior.

## Global constraints

- All local TMPDIR/scratch is /Volumes/KIOXIA/Developments/tmp; source/evidence on external disks. Boot APFS >80% forbids local Cargo/R execution.
- Original red-7/green-11 snapshots and uploaded ZIPs are immutable; never rewrite, filter or borrow files to accept them.
- checksums.sha256 contains exactly README.md, files.sha256, source.tar.gz, sorted and LF-terminated.
- Every logical tar member has one safe canonical path, exactly REGTYPE and its manifest hash. No undeclared AppleDouble or arbitrary extra files.
- Tarfile's legal PAX/GNU representation metadata remains allowed; no private Python tar API or physical-header parser.
- New guard does not replace trusted receipt identity, four-native CI, Cargo policy or post-trial source checks.
- No production Rust, lock, existing fixtures, old bench or old support bytes change.

## Review focus

- A checksum missing README must fail even when archive/manifest hashes match: explicit two-entry checksum regression.
- AppleDouble must not be silently filtered: root/inner/ZIP-style metadata names fail exact logical inventory.
- A later member byte failure must not leave extracted source: verify all members before mkdir, mutation test asserts absent output.
- Duplicate/unsafe/link/special/legacy logical members must fail before output creation: complete controlled tar mutations.
- Extraction into existing/symlink paths must preserve them; old accepted measurement PAX archives remain compatible.

---

### Task 1: Test-first source guard

**Files:** Create scripts/test_infercnv_source_snapshot.py.

**Interfaces:** imports _manifest(bytes)->dict, read_snapshot_manifest(Path)->dict, verify_and_extract(snapshot_dir, output_dir, manifest=None)->dict; these functions do not exist until the observed RED.

- [x] **Step 1: Add the complete contract tests with the helper absent.**

```python
import hashlib
import io
from pathlib import Path
import sys
import tarfile
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
from infercnv_source_snapshot import _manifest, read_snapshot_manifest, verify_and_extract


def digest(data):
    return hashlib.sha256(data).hexdigest()


def manifest_bytes(files):
    return b"".join(f"{digest(data)}  {name}\n".encode() for name, data in sorted(files.items()))


class SourceSnapshotTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.snapshot = self.root / "snapshot"
        self.snapshot.mkdir()
        self.output = self.root / "output"
        self.files = {"Cargo.lock": b"lock\n", "Cargo.toml": b"package\n", "src/lib.rs": b"production\n"}
        self.make_snapshot()

    def make_checks(self, names=("README.md", "files.sha256", "source.tar.gz")):
        checks = b"".join(f"{digest((self.snapshot / name).read_bytes())}  {name}\n".encode() for name in names)
        (self.snapshot / "checksums.sha256").write_bytes(checks)

    def make_snapshot(self, members=None, archive_format=tarfile.USTAR_FORMAT):
        (self.snapshot / "files.sha256").write_bytes(manifest_bytes(self.files))
        (self.snapshot / "README.md").write_bytes(b"snapshot description\n")
        with tarfile.open(self.snapshot / "source.tar.gz", "w:gz", format=archive_format) as archive:
            for name, data, kind in members if members is not None else [(n, d, tarfile.REGTYPE) for n, d in self.files.items()]:
                info = tarfile.TarInfo(name)
                info.type = kind
                info.size = len(data) if kind in (tarfile.REGTYPE, tarfile.AREGTYPE, tarfile.CONTTYPE) else 0
                if kind in (tarfile.SYMTYPE, tarfile.LNKTYPE):
                    info.linkname = "Cargo.lock"
                if archive_format == tarfile.PAX_FORMAT:
                    info.pax_headers = {"mtime": "1.5"}
                archive.addfile(info, io.BytesIO(data) if info.size else None)
        self.make_checks()

    def rejected(self, pattern):
        with self.assertRaisesRegex(ValueError, pattern):
            verify_and_extract(self.snapshot, self.output)
        self.assertFalse(self.output.exists())

    def test_exact_ustar_snapshot(self):
        expected = {name: digest(data) for name, data in self.files.items()}
        self.assertEqual(read_snapshot_manifest(self.snapshot), expected)
        self.assertEqual(verify_and_extract(self.snapshot, self.output), expected)
        self.assertEqual({p.relative_to(self.output).as_posix(): p.read_bytes()
                          for p in self.output.rglob("*") if p.is_file()}, self.files)

    def test_pax_representation_preserves_logical_contract(self):
        self.make_snapshot(archive_format=tarfile.PAX_FORMAT)
        self.assertEqual(set(verify_and_extract(self.snapshot, self.output)), set(self.files))

    def test_checksum_must_include_readme(self):
        self.make_checks(("files.sha256", "source.tar.gz"))
        self.rejected("snapshot checksums mismatch")

    def test_checksum_cannot_include_extra(self):
        (self.snapshot / "extra").write_bytes(b"extra")
        self.make_checks(("README.md", "extra", "files.sha256", "source.tar.gz"))
        self.rejected("snapshot checksums mismatch")

    def test_checksum_readme_bytes_must_match(self):
        (self.snapshot / "README.md").write_bytes(b"changed description\n")
        self.rejected("snapshot checksums mismatch")

    def test_duplicate_checksum_path(self):
        path = self.snapshot / "checksums.sha256"
        path.write_bytes(path.read_bytes().splitlines(keepends=True)[0] + path.read_bytes())
        self.rejected("duplicate manifest path")

    def test_appledouble_and_other_extra_regular_members(self):
        complete = [(n, d, tarfile.REGTYPE) for n, d in self.files.items()]
        for name in ("._Cargo.toml", "src/._lib.rs", "__MACOSX/._Cargo.lock", "unlisted.rs"):
            with self.subTest(name=name):
                self.make_snapshot(complete + [(name, b"extra", tarfile.REGTYPE)])
                self.rejected("archive names/types")

    def test_missing_logical_member(self):
        self.make_snapshot([(n, d, tarfile.REGTYPE) for n, d in self.files.items() if n != "src/lib.rs"])
        self.rejected("archive names/types")

    def test_duplicate_logical_member(self):
        complete = [(n, d, tarfile.REGTYPE) for n, d in self.files.items()]
        self.make_snapshot(complete + [("Cargo.lock", b"again", tarfile.REGTYPE)])
        self.rejected("duplicate path")

    def test_unsafe_member_names(self):
        complete = [(n, d, tarfile.REGTYPE) for n, d in self.files.items()]
        for name in ("../bad", "/bad", "src/../bad", "src//bad", "./bad", "C:bad", "src\\bad", "bad\tname"):
            with self.subTest(name=name):
                self.make_snapshot(complete + [(name, b"bad", tarfile.REGTYPE)])
                self.rejected("unsafe path")

    def test_file_directory_collision(self):
        complete = [(n, d, tarfile.REGTYPE) for n, d in self.files.items()]
        self.make_snapshot(complete + [("src", b"bad", tarfile.REGTYPE)])
        self.rejected("file/directory collision")

    def test_nonregular_logical_member_types(self):
        for kind in (tarfile.SYMTYPE, tarfile.LNKTYPE, tarfile.DIRTYPE, tarfile.FIFOTYPE,
                     tarfile.CHRTYPE, tarfile.BLKTYPE, tarfile.AREGTYPE, tarfile.CONTTYPE):
            with self.subTest(kind=kind):
                self.make_snapshot([(n, d, kind if n == "src/lib.rs" else tarfile.REGTYPE)
                                    for n, d in self.files.items()])
                self.rejected("archive names/types")

    def test_member_hash_failure_precedes_output_creation(self):
        self.make_snapshot([(n, b"wrong bytes" if n == "src/lib.rs" else d, tarfile.REGTYPE)
                            for n, d in self.files.items()])
        self.rejected("archive hash mismatch: src/lib.rs")

    def test_manifest_changed_between_selection_and_extraction(self):
        expected = read_snapshot_manifest(self.snapshot)
        self.files["src/lib.rs"] = b"other valid snapshot\n"
        self.make_snapshot()
        with self.assertRaisesRegex(ValueError, "source manifest changed"):
            verify_and_extract(self.snapshot, self.output, expected)
        self.assertFalse(self.output.exists())

    def test_symlinked_snapshot_file(self):
        original = self.snapshot / "README.md"
        backup = self.root / "readme"
        original.rename(backup)
        original.symlink_to(backup)
        self.rejected("not a regular file")

    def test_symlinked_snapshot_directory(self):
        alias = self.root / "alias"
        alias.symlink_to(self.snapshot, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "symlinked path"):
            verify_and_extract(alias, self.output)
        self.assertFalse(self.output.exists())

    def test_existing_output_and_symlink_output(self):
        self.output.mkdir()
        with self.assertRaisesRegex(ValueError, "output directory already exists"):
            verify_and_extract(self.snapshot, self.output)
        broken = self.root / "broken"
        broken.symlink_to(self.root / "absent")
        with self.assertRaisesRegex(ValueError, "output directory already exists"):
            verify_and_extract(self.snapshot, broken)

    def test_output_parent_symlink(self):
        parent = self.root / "parent"
        parent.symlink_to(self.snapshot, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "symlinked path"):
            verify_and_extract(self.snapshot, parent / "output")

    def test_manifest_spelling_and_inventory(self):
        good = manifest_bytes(self.files)
        first = good.splitlines(keepends=True)[0]
        bad_cases = ((b"", "LF-terminated"), (good.rstrip(b"\n"), "LF-terminated"),
                     (good.replace(b"\n", b"\r\n"), "LF-terminated"),
                     (b"\xff\n", "UTF-8"), (good.replace(b"  ", b" *", 1), "malformed manifest row"),
                     (b"x" * 64 + b"  Cargo.lock\n", "malformed manifest row"),
                     (first + good, "duplicate manifest path"),
                     (b"".join(reversed(good.splitlines(keepends=True))), "empty or unsorted"))
        for data, pattern in bad_cases:
            with self.subTest(data=data):
                with self.assertRaisesRegex(ValueError, pattern):
                    _manifest(data)
        for names in (("src", "src/lib.rs"), ("../evil",), ("e\u0301",)):
            with self.subTest(names=names):
                data = b"".join(f"{'0' * 64}  {name}\n".encode() for name in sorted(names))
                with self.assertRaises(ValueError):
                    _manifest(data)


if __name__ == "__main__":
    unittest.main()
```

- [x] **Step 2: Execute and preserve genuine RED.**

```bash
TMPDIR=/Volumes/KIOXIA/Developments/tmp PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest discover -s scripts -p test_infercnv_source_snapshot.py -v
```

Expected ModuleNotFoundError naming infercnv_source_snapshot. Existing source snapshot tests are still unchanged and pass independently. Do not accept generic environment/syntax failure.

### Task 2: Shared implementation and both concrete consumers

**Files:** Create scripts/infercnv_source_snapshot.py; modify scripts/verify_infercnv_measurement_snapshot.py and .github/workflows/sc-cnv-core-candidate.yml; add the new imported module to .github/workflows/infercnv-oracle.yml existing measurement-code-sha256.txt command.

**Interfaces:** generic guard returns exact source manifest and creates exclusive validated source output. Existing receipt verifier keeps verify_snapshot/verify_extracted/record_extracted_check and receipt schema unchanged. Candidate step preserves its name and runs this same guard before Rust setup.

- [x] **Step 1: Add the complete shared helper after RED.**

```python
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
```

- [x] **Step 2: Apply the complete caller edits below.**

# Logical source snapshot guard proposal

Proposed destination files are `scripts/infercnv_source_snapshot.py` and
`scripts/test_infercnv_source_snapshot.py`. The complete files in this directory
are drafts, not executed tests or accepted repository implementation.

The helper preserves the existing verifier's logical tar contract. PAX/GNU
representation metadata is handled by Python's `tarfile`; every returned
logical source member must be exactly `REGTYPE`, declared once, safely named,
and match its manifest SHA256. AppleDouble is not filtered or silently removed:
its undeclared logical members fail the gate. `checksums.sha256` must bind
exactly README, source manifest and source archive. All member hashes are checked
before output directory creation, then checked again while extracting through
the same open TarFile. No `extractall`, shell tar or Python private tar API.

## Existing measurement verifier integration

In `scripts/verify_infercnv_measurement_snapshot.py`:

1. Remove unused imports `hashlib`, `PurePosixPath`, `stat`, `unicodedata`.
   Keep `Path`, `re`, `tarfile` and all receipt/run/Cargo checks.
2. Replace the implementations of `_hash_file`, `_regular`, `_no_symlink_path`,
   `_name`, `_names_without_collisions` and `_manifest` with this import:

```python
from infercnv_source_snapshot import (
    _hash_file, _regular, _no_symlink_path, _name,
    _names_without_collisions, _manifest,
    read_snapshot_manifest, verify_and_extract,
)
```

`HASH` can either stay receipt-local (same regex, not a second implementation)
or be imported from the shared helper. Receipt `_receipt`, `_run_jobs`,
`_check_cargo`, post-trial `verify_extracted` and diagnostic receipt functions
remain unchanged.

3. In `verify_snapshot`, preserve receipt/run/path checks, absent-output checks
   and archive/manifest hashes against the trusted receipt. Replace the block
   from `for name in ("README.md", ...)` through source manifest parsing with:

```python
manifest = read_snapshot_manifest(snapshot_dir)
if _hash_file(snapshot_dir / "source.tar.gz") != receipt["archive_sha256"] or \
        _hash_file(snapshot_dir / "files.sha256") != receipt["manifest_sha256"]:
    raise ValueError("snapshot digest mismatch")
```

4. Keep the existing policy block requiring Cargo.lock/Cargo.toml, prohibiting
   source `files.sha256`, comparing Cargo.lock hash, and comparing the complete
   `src/` manifest with `production_sha256`. Replace the entire old
   `with tarfile.open(...)` extraction block with:

```python
verify_and_extract(snapshot_dir, output_dir, manifest)
```

5. Keep the existing final `verify_extracted(...)` and `return output_dir`.
   Thus a generic source guard does not replace measured-support Cargo policy,
   four-native run identity, trusted receipt binding or post-trial rechecks.

## Candidate workflow integration

In `.github/workflows/sc-cnv-core-candidate.yml` replace only the initial
checksum subshell, explicit output mkdir and `tar -xzf` block with:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest discover \
  -s scripts -p test_infercnv_source_snapshot.py \
  2>&1 | tee "$EVIDENCE/source-guard-tests.log"
PYTHONDONTWRITEBYTECODE=1 python3 -B scripts/infercnv_source_snapshot.py \
  --snapshot-dir "$snapshot" \
  --output-dir "$RUNNER_TEMP/sc-cnv-candidate" \
  2>&1 | tee "$EVIDENCE/archive-verification.log"
```

Keep the `Verify and extract frozen source` step name, native identity checks,
output environment, copied original archive/README/manifests and existing
source-before/source-after evidence. Shell `set -euo pipefail` is already set;
failed tests or guard must stop before Rust installation/compilation. No RED
selector may bypass this packaging guard. Green-11/red-7 failures remain
preserved as historical packaging failures, not algorithm regressions.

The future uploaded evidence should include both shared guard/test source files
or bind them to the exact control-plane head already recorded in host.log.

## Test-first integration gates

- Copy only the proposed tests first; run them with the new helper absent and
  preserve the concrete ModuleNotFoundError RED output.
- Add the shared helper and the two existing callers' integrations.
- Run the 19 new test methods, then the existing measurement snapshot suite.
  Preserve complete logs and final real counts; no count here is a pass claim.
- Confirm actual old accepted measurement snapshots retain their existing
  receipt/Cargo acceptance, including any legal PAX representation.
- Confirm original polluted green-11 is rejected for extra logical members;
  independently, the original two-entry checksums are rejected for missing
  README. Neither original archive nor checksum file may be rewritten.
- Run the candidate guard on newly generated clean green-12 before dispatch.
  This is a packaging gate only; unchanged production hashes, frozen support,
  all four native executions and original raw logs remain separately required.

External local test command after integration:

```bash
TMPDIR=/Volumes/KIOXIA/Developments/tmp PYTHONDONTWRITEBYTECODE=1 \
  python3 -B -m unittest discover -s scripts -p test_infercnv_source_snapshot.py -v
TMPDIR=/Volumes/KIOXIA/Developments/tmp PYTHONDONTWRITEBYTECODE=1 \
  python3 -B -m unittest discover -s scripts -p test_verify_infercnv_measurement_snapshot.py -v
```

No Rust build, R execution, dependency install, archive repair, dispatch or
performance claim belongs to this proposal.


For candidate raw evidence also copy scripts/infercnv_source_snapshot.py and its test to EVIDENCE/source-guard and SHA256 both original files, so future offline review can authenticate executed guard code. The exact additional Bash is:

```bash
mkdir "$EVIDENCE/source-guard"
cp scripts/infercnv_source_snapshot.py scripts/test_infercnv_source_snapshot.py "$EVIDENCE/source-guard/"
(cd "$EVIDENCE/source-guard"; shasum -a 256 infercnv_source_snapshot.py test_infercnv_source_snapshot.py) > "$EVIDENCE/source-guard/checksums.sha256"
```

In the existing oracle measurement code SHA256 command add scripts/infercnv_source_snapshot.py immediately after scripts/verify_infercnv_measurement_snapshot.py. No source receipt, measured command, timer or benchmark changes.

- [x] **Step 3: Run focused GREEN, old verifier regressions and complete controller suite.**

```bash
TMPDIR=/Volumes/KIOXIA/Developments/tmp PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest discover -s scripts -p test_infercnv_source_snapshot.py -v
TMPDIR=/Volumes/KIOXIA/Developments/tmp PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest discover -s scripts -p test_verify_infercnv_measurement_snapshot.py -v
TMPDIR=/Volumes/KIOXIA/Developments/tmp PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest discover -s scripts -p 'test_*.py'
python3 -B scripts/validate_control_plane.py
git diff --check
```

Expected 19 new source-guard tests and all old verifier tests pass; complete suite derives its actual count from raw output (223 existing +19 new =242 expected). Read any failure; never skip inherited failures. Ruby YAML parsing and Bash syntax checks cover both actual modified workflows. Fresh independent source/integration review must retain old receipt/run/Cargo rejection behavior.

### Task 3: Actual immutable snapshots and fresh four-native candidate

**Files:** Clean green-12 README/archive/manifest/three-entry checksums; owned measurement ledger and this plan.

**Interfaces:** shared guard rejects originals unchanged and extracts newly packaged byte-identical green-12; new native job authenticates guard and Rust source independently.

- [x] **Step 1: Prove actual old accepted source compatibility and new source rejection/acceptance.**

Run the guard on accepted green-5 into a new external temporary output, check exact manifest files. Original green-11 must fail because its two-entry checksum omits README. Independently run its original tar through the shared _verified_members using its original manifest to observe undeclared logical members; do not rewrite its checksums to reach that test. Clean green-12 must pass with exactly146 logical sources/hashes. All extraction outputs are new external mktemp directories; no deletion or reuse of evidence.

Root also verified the active prepared-measurement receipt's green-7 using its
original saved native-run/jobs APIs through the complete updated verify_snapshot
entry point. Receipt/run/Cargo/production checks and extraction passed unchanged.
Original green-11 checksums fail first because their rows are unsorted; the
separate sorted two-entry regression proves missing README rejection. Its
original tar independently fails the logical-member inventory. Originals remain
unchanged. Root observed 19 focused, 12 old verifier and 242 total controller
tests passing, plus YAML and all 12/22 Bash blocks in the two modified workflows.

- [ ] **Step 2: Commit/push only owned guard/caller/plan/state/green-12 files and verify exact-head Control plane CI.**

Commit subject: fix(sc): enforce frozen source inventory. Preserve unrelated VCF/scratch and the uncommitted owning product. Dispatch only after pushed head succeeds:

```bash
gh workflow run sc-cnv-core-candidate.yml --repo omics-rust/rsomics-world --ref main -f snapshot=green-12 -f expected_red=false -f measurement=true -f ingestion_measurement=true -f oracle_receipt=infercnv-shipped-2026-09-26
```

- [ ] **Step 3: Audit original four-native APIs/logs/artifacts before scoped Rust-support acceptance.**

Require the same unchanged 55 creation-support tests, 11 old measurement tests, 72 ordinary/external tests per debug/release/target, old bench checks and Linux format/strict Clippy. Require19 guard tests and exact copied guard hashes on each target, native host/rustc/output path, original safe ZIP/CRC/API hashes, exactly146 logical source members, three checksum hashes, source-before/after equality and unchanged lock/production. Preserve failed packaging runs as such. No factory timing, speed/resource result or whole inferCNV/public release follows this gate.
