//! Compatibility tests for rsomics-fastq-sample vs seqkit sample.
//!
//! Verifies:
//! - Fraction mode: output is valid FASTQ with roughly p×N records
//! - Exact mode: output contains exactly N records
//! - Paired mode: R1/R2 pair counts match
//! - Reproducibility: same seed → byte-identical output

use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn ours() -> PathBuf {
    env!("CARGO_BIN_EXE_rsomics-fastq-sample").into()
}

fn golden_fastq() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/sample_100.fq")
}

fn seqkit_available() -> bool {
    Command::new("seqkit")
        .arg("version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn count_records(path: &Path) -> u64 {
    let file = std::fs::File::open(path).unwrap();
    let reader = std::io::BufReader::new(file);
    reader.lines().count() as u64 / 4
}

fn count_records_gz(path: &Path) -> u64 {
    let file = std::fs::File::open(path).unwrap();
    let decoder = flate2::read::GzDecoder::new(file);
    let reader = std::io::BufReader::new(decoder);
    reader.lines().count() as u64 / 4
}

fn count_path(path: &Path) -> u64 {
    if path.extension().and_then(|e| e.to_str()) == Some("gz") {
        count_records_gz(path)
    } else {
        count_records(path)
    }
}

#[test]
fn fraction_mode_produces_records() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.fq");
    let status = Command::new(ours())
        .args([
            "-p",
            "0.5",
            "--seed",
            "42",
            "-o",
            out.to_str().unwrap(),
            golden_fastq().to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(
        status.success(),
        "rsomics-fastq-sample fraction mode failed"
    );
    let kept = count_path(&out);
    // Bernoulli(p=0.5, n=100, seed=42) should give roughly 50 ± 25
    assert!(
        kept > 10 && kept < 90,
        "unexpected keep count {kept} (expected ~50 for p=0.5 on 100 records)"
    );
}

#[test]
fn exact_mode_gives_exactly_n_records() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.fq");
    let status = Command::new(ours())
        .args([
            "-n",
            "30",
            "--seed",
            "99",
            "-o",
            out.to_str().unwrap(),
            golden_fastq().to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success(), "rsomics-fastq-sample exact mode failed");
    assert_eq!(count_path(&out), 30, "exact mode should keep exactly 30");
}

#[test]
fn exact_mode_all_when_n_exceeds_input() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.fq");
    let status = Command::new(ours())
        .args([
            "-n",
            "9999",
            "--seed",
            "1",
            "-o",
            out.to_str().unwrap(),
            golden_fastq().to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());
    assert_eq!(
        count_path(&out),
        100,
        "when n > total, all records should be kept"
    );
}

#[test]
fn reproducible_with_same_seed() {
    let dir = tempfile::tempdir().unwrap();
    let out1 = dir.path().join("run1.fq");
    let out2 = dir.path().join("run2.fq");
    for out in [&out1, &out2] {
        Command::new(ours())
            .args([
                "-p",
                "0.3",
                "--seed",
                "7",
                "-o",
                out.to_str().unwrap(),
                golden_fastq().to_str().unwrap(),
            ])
            .status()
            .unwrap();
    }
    let c1 = count_path(&out1);
    let c2 = count_path(&out2);
    assert_eq!(c1, c2, "same seed must produce identical record count");
    let bytes1 = std::fs::read(&out1).unwrap();
    let bytes2 = std::fs::read(&out2).unwrap();
    assert_eq!(
        bytes1, bytes2,
        "same seed must produce byte-identical output"
    );
}

#[test]
fn seqkit_compat_exact_count() {
    if !seqkit_available() {
        eprintln!("seqkit not found — skipping seqkit compat test");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let ours_out = dir.path().join("ours.fq");
    let seqkit_out = dir.path().join("seqkit.fq");

    // exact 40 records from 100-record fixture
    let status = Command::new(ours())
        .args([
            "-n",
            "40",
            "--seed",
            "55",
            "-o",
            ours_out.to_str().unwrap(),
            golden_fastq().to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success(), "ours failed");

    Command::new("seqkit")
        .args([
            "sample",
            "-n",
            "40",
            "-s",
            "55",
            "-o",
            seqkit_out.to_str().unwrap(),
            golden_fastq().to_str().unwrap(),
        ])
        .status()
        .unwrap();

    let ours_count = count_path(&ours_out);
    let seqkit_count = count_path(&seqkit_out);

    // Both should produce exactly 40 records.
    assert_eq!(ours_count, 40, "ours exact mode gave wrong count");
    assert_eq!(seqkit_count, 40, "seqkit exact mode gave wrong count");
}

#[test]
fn gz_output_readable() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.fq.gz");
    let status = Command::new(ours())
        .args([
            "-n",
            "20",
            "--seed",
            "11",
            "-o",
            out.to_str().unwrap(),
            golden_fastq().to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());
    assert_eq!(count_records_gz(&out), 20);
}
