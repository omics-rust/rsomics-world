//! Smoke tests: verify basic functionality without requiring Picard.

use std::path::Path;
use std::process::Command;

const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");
const BIN: &str = env!("CARGO_BIN_EXE_rsomics-rnaseq-metrics");

#[test]
fn smoke_no_rrna() {
    let golden = Path::new(GOLDEN);
    let out = tempfile::NamedTempFile::new().expect("tempfile");
    let status = Command::new(BIN)
        .args([
            "--input",
            golden.join("test.bam").to_str().unwrap(),
            "--output",
            out.path().to_str().unwrap(),
            "--ref-flat",
            golden.join("test.refFlat").to_str().unwrap(),
            "--strand-specificity",
            "NONE",
        ])
        .output()
        .expect("running binary");

    assert!(
        status.status.success(),
        "binary failed: {}",
        String::from_utf8_lossy(&status.stderr)
    );

    let content = std::fs::read_to_string(out.path()).expect("reading output");
    assert!(
        content.contains("## METRICS CLASS"),
        "missing Picard header"
    );
    assert!(content.contains("PF_BASES"), "missing field header");
    // Verify key counts appear in output.
    assert!(content.contains("496"), "expected CODING_BASES=496");
    assert!(content.contains("332"), "expected UTR_BASES=332");
    assert!(content.contains("101"), "expected INTRONIC_BASES=101");
    assert!(content.contains("151"), "expected INTERGENIC_BASES=151");
}

#[test]
fn smoke_with_rrna() {
    let golden = Path::new(GOLDEN);
    let out = tempfile::NamedTempFile::new().expect("tempfile");
    let status = Command::new(BIN)
        .args([
            "--input",
            golden.join("test.bam").to_str().unwrap(),
            "--output",
            out.path().to_str().unwrap(),
            "--ref-flat",
            golden.join("test.refFlat").to_str().unwrap(),
            "--ribosomal-intervals",
            golden.join("rrna.interval_list").to_str().unwrap(),
            "--strand-specificity",
            "NONE",
        ])
        .output()
        .expect("running binary");

    assert!(
        status.status.success(),
        "binary failed: {}",
        String::from_utf8_lossy(&status.stderr)
    );

    let content = std::fs::read_to_string(out.path()).expect("reading output");
    // Verify rRNA counts appear.
    assert!(content.contains("200"), "expected RIBOSOMAL_BASES=200");
    assert!(content.contains("477"), "expected CODING_BASES=477");
    assert!(content.contains("202"), "expected UTR_BASES=202");
}

#[test]
fn smoke_missing_bam() {
    let out = tempfile::NamedTempFile::new().expect("tempfile");
    let status = Command::new(BIN)
        .args([
            "--input",
            "/nonexistent/file.bam",
            "--output",
            out.path().to_str().unwrap(),
            "--ref-flat",
            "/nonexistent/file.refFlat",
        ])
        .output()
        .expect("running binary");

    assert!(
        !status.status.success(),
        "expected failure for missing input"
    );
}
