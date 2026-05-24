//! Smoke tests for rsomics-inner-distance against the golden fixture.

use std::path::Path;
use std::process::Command;

const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");
const BIN: &str = env!("CARGO_BIN_EXE_rsomics-inner-distance");

fn golden_bam() -> std::path::PathBuf {
    Path::new(GOLDEN).join("pairs.bam")
}

fn golden_bed() -> std::path::PathBuf {
    Path::new(GOLDEN).join("genes.bed12")
}

#[test]
fn runs_on_golden_fixture() {
    let bam = golden_bam();
    let bed = golden_bed();
    if !bam.exists() || !bed.exists() {
        eprintln!("SKIP: golden fixture not found (run tests/make_golden.py first)");
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let prefix = tmp.path().join("smoke").to_string_lossy().into_owned();

    let out = Command::new(BIN)
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-r",
            bed.to_str().unwrap(),
            "-o",
            &prefix,
            "--mapq",
            "30",
            "-l",
            "-250",
            "-u",
            "250",
            "-s",
            "5",
        ])
        .output()
        .expect("failed to run rsomics-inner-distance");

    assert!(
        out.status.success(),
        "rsomics-inner-distance failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let freq_path = format!("{prefix}.inner_distance_freq.txt");
    let freq = std::fs::read_to_string(&freq_path)
        .unwrap_or_else(|e| panic!("failed to read {freq_path}: {e}"));

    // Histogram must have (250 - (-250)) / 5 = 100 lines.
    let lines: Vec<&str> = freq.lines().collect();
    assert_eq!(lines.len(), 100, "histogram should have 100 bins");

    // First bin header: -250\t-245\t0
    assert!(
        lines[0].starts_with("-250\t-245\t"),
        "first bin should be -250..-245"
    );

    // Last bin: 245\t250\tN
    assert!(
        lines[99].starts_with("245\t250\t"),
        "last bin should be 245..250"
    );
}

#[test]
fn per_pair_output_has_expected_rows() {
    let bam = golden_bam();
    let bed = golden_bed();
    if !bam.exists() || !bed.exists() {
        eprintln!("SKIP: golden fixture not found");
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let prefix = tmp.path().join("pairs_test").to_string_lossy().into_owned();

    let out = Command::new(BIN)
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-r",
            bed.to_str().unwrap(),
            "-o",
            &prefix,
            "--mapq",
            "30",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());

    let txt_path = format!("{prefix}.inner_distance.txt");
    let txt = std::fs::read_to_string(&txt_path).unwrap();
    let rows: Vec<Vec<&str>> = txt.lines().map(|l| l.split('\t').collect()).collect();

    // 4 pairs in the fixture.
    assert_eq!(rows.len(), 4, "expected 4 pair rows, got {}", rows.len());

    // pair1: 50, sameExon=Yes
    let pair1 = rows
        .iter()
        .find(|r| r[0] == "pair1")
        .expect("pair1 not found");
    assert_eq!(pair1[1], "50", "pair1 distance");
    assert_eq!(pair1[2], "sameTranscript=Yes,sameExon=Yes,dist=mRNA");

    // pair2: 50, sameExon=No
    let pair2 = rows
        .iter()
        .find(|r| r[0] == "pair2")
        .expect("pair2 not found");
    assert_eq!(pair2[1], "50", "pair2 distance");
    assert_eq!(pair2[2], "sameTranscript=Yes,sameExon=No,dist=mRNA");

    // pair3: -100, readPairOverlap
    let pair3 = rows
        .iter()
        .find(|r| r[0] == "pair3")
        .expect("pair3 not found");
    assert_eq!(pair3[1], "-100", "pair3 distance");
    assert_eq!(pair3[2], "readPairOverlap");

    // pair4: 100, sameTranscript=No
    let pair4 = rows
        .iter()
        .find(|r| r[0] == "pair4")
        .expect("pair4 not found");
    assert_eq!(pair4[1], "100", "pair4 distance");
    assert_eq!(pair4[2], "sameTranscript=No,dist=genomic");
}
