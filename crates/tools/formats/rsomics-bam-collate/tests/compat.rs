//! Compat invariants for `rsomics-bam-collate` against `samtools collate`.
//!
//! Collation has no canonical inter-group order: samtools collate emits groups
//! in an internal hash order (`bamshuf.c`), ours in first-seen-QNAME order. Both
//! are valid collations, so byte-equality to samtools is NOT the contract — the
//! invariants are. This test asserts:
//!
//!   1. multiset of records is preserved (every input record present exactly
//!      once in the output, by full-SAM-line key);
//!   2. all records sharing a QNAME are contiguous in the output;
//!   3. ours is deterministic (same input → byte-identical output across runs).
//!
//! As a cross-check, samtools collate run on the same input is asserted to
//! satisfy invariants (1) and (2) as well — confirming the invariants are the
//! real spec both implementations meet, not an artefact of ours.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn ours() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-collate"))
}

fn golden() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/scrambled.bam")
}

fn samtools_available() -> bool {
    Command::new("samtools")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn run_ok(cmd: &mut Command) {
    let out = cmd.output().unwrap();
    assert!(
        out.status.success(),
        "command failed: {cmd:?}\nstderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Every alignment record decoded to a SAM line, in stream order. Each line is a
/// stable per-record key (flags + all fields), so primary vs supplementary of
/// the same QNAME are distinct keys.
fn records_in_order(bam: &Path) -> Vec<String> {
    let out = Command::new("samtools")
        .arg("view")
        .arg(bam)
        .output()
        .unwrap();
    assert!(out.status.success());
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_owned)
        .collect()
}

/// QNAME (column 1) sequence of the records, in output order.
fn qnames_in_order(bam: &Path) -> Vec<String> {
    records_in_order(bam)
        .into_iter()
        .map(|l| l.split('\t').next().unwrap().to_owned())
        .collect()
}

/// Invariant 1: the output's record multiset equals the input's.
fn assert_multiset_preserved(input: &Path, output: &Path, who: &str) {
    let mut a = records_in_order(input);
    let mut b = records_in_order(output);
    a.sort();
    b.sort();
    assert_eq!(a, b, "[{who}] output multiset must equal input multiset");
}

/// Invariant 2: every QNAME occupies a single contiguous run in the output.
fn assert_qnames_contiguous(output: &Path, who: &str) {
    let qnames = qnames_in_order(output);
    let mut seen = std::collections::HashSet::new();
    let mut prev: Option<&String> = None;
    for q in &qnames {
        if Some(q) != prev {
            assert!(
                seen.insert(q.clone()),
                "[{who}] QNAME {q} is split into non-contiguous runs: {qnames:?}"
            );
            prev = Some(q);
        }
    }
}

#[test]
fn collate_invariants_hold_and_match_samtools_spec() {
    if !samtools_available() {
        eprintln!("skipping: samtools not found");
        return;
    }
    let dir = std::env::temp_dir().join("rsomics-bam-collate-compat");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let input = golden();

    // ours
    let o1 = dir.join("ours1.bam");
    run_ok(ours().arg(&input).arg("-o").arg(&o1));
    assert_multiset_preserved(&input, &o1, "ours");
    assert_qnames_contiguous(&o1, "ours");

    // invariant 3: determinism — a second run is byte-identical.
    let o2 = dir.join("ours2.bam");
    run_ok(ours().arg(&input).arg("-o").arg(&o2));
    let b1 = std::fs::read(&o1).unwrap();
    let b2 = std::fs::read(&o2).unwrap();
    assert_eq!(
        b1, b2,
        "ours must be deterministic: byte-identical across runs"
    );

    // cross-check: samtools collate meets the same invariants on this input.
    let st = dir.join("samtools.bam");
    run_ok(
        Command::new("samtools")
            .arg("collate")
            .arg("-o")
            .arg(&st)
            .arg(&input),
    );
    assert_multiset_preserved(&input, &st, "samtools");
    assert_qnames_contiguous(&st, "samtools");

    let _ = std::fs::remove_dir_all(&dir);
}
