"""Verify four-native raw-creation tests and the frozen private trial source."""

import argparse
from pathlib import Path
import tarfile
import tomllib

import verify_infercnv_measurement_snapshot as prepared


RAW_STEPS = ("Test ingestion trial debug", "Test ingestion trial release and compile bench",
             "Test ingestion support debug", "Test ingestion support release")


def _raw_jobs(receipt, run, jobs):
    prepared._run_jobs(receipt, run, jobs)
    for job in jobs["jobs"]:
        for name in RAW_STEPS:
            records = [step for step in job["steps"] if step.get("name") == name]
            if len(records) != 1 or records[0].get("status") != "completed" or records[0].get("conclusion") != "success":
                raise ValueError(f"raw creation job {job['name']} requires successful unique step: {name}")
        external = next(step for step in job["steps"] if step.get("name") == "Acquire and verify pinned external oracle")
        if external["conclusion"] != "success":
            raise ValueError("raw creation requires the pinned external oracle on all targets")


def _raw_cargo(root):
    try:
        cargo = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    except (OSError, UnicodeError, tomllib.TOMLDecodeError) as exc:
        raise ValueError("invalid raw creation Cargo.toml") from exc
    expected = {
        "bench": (("cnv_ingestion_matched", "benches/cnv_ingestion_matched.rs"),),
        "test": (("cnv_ingestion_trial_contract", "tests/cnv_ingestion_trial_contract.rs"),
                 ("cnv_ingestion_measurement_contract", "tests/cnv_ingestion_measurement_contract.rs")),
    }
    for kind, targets in expected.items():
        collection = cargo.get(kind, [])
        if not isinstance(collection, list) or any(not isinstance(entry, dict) for entry in collection):
            raise ValueError(f"raw creation Cargo target collection mismatch: {kind}")
        for name, path in targets:
            entries = [entry for entry in collection if entry.get("name") == name]
            if len(entries) != 1 or entries[0].get("path") != path or \
                    entries[0].get("required-features") != ["infercnv-measurement"] or \
                    kind == "bench" and (entries[0].get("harness") is not False or entries[0].get("test") is not False):
                raise ValueError(f"raw creation Cargo target mismatch: {name}")
            prepared._regular(root / path)


def verify_ingestion_snapshot(receipt_path, run_path, jobs_path, snapshot_dir, output_dir):
    receipt = prepared._read_json(receipt_path)
    prepared._receipt(receipt)
    _raw_jobs(receipt, prepared._read_json(run_path), prepared._read_json(jobs_path))
    root = prepared.verify_snapshot(receipt_path, run_path, jobs_path, snapshot_dir, output_dir)
    _raw_cargo(root)
    return root


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("receipt", "run-json", "jobs-json", "snapshot-dir", "output-dir"):
        parser.add_argument("--" + name, required=True, type=Path)
    args = parser.parse_args()
    try:
        root = verify_ingestion_snapshot(args.receipt, args.run_json, args.jobs_json, args.snapshot_dir, args.output_dir)
    except (OSError, ValueError, tarfile.TarError) as exc:
        parser.exit(1, f"raw creation snapshot verification failed: {exc}\n")
    print(f"verified raw creation source: {root}")


if __name__ == "__main__":
    main()
