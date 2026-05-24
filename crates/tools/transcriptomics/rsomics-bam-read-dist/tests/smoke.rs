//! Smoke tests: run the binary on golden fixtures and check expected output fields.

use std::path::Path;
use std::process::Command;

const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");

fn run_bin(bam: &str, bed: &str) -> String {
    let our_bin = env!("CARGO_BIN_EXE_rsomics-bam-read-dist");
    let out = Command::new(our_bin)
        .args(["-i", bam, "-r", bed, "-t", "1"])
        .output()
        .expect("failed to run rsomics-bam-read-dist");
    assert!(
        out.status.success(),
        "binary failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn smoke_golden_counts() {
    let bam = Path::new(GOLDEN).join("reads.bam");
    let bed = Path::new(GOLDEN).join("genes.bed12");
    let stdout = run_bin(bam.to_str().unwrap(), bed.to_str().unwrap());

    assert!(
        stdout.contains("Total Reads                   73"),
        "total reads"
    );
    assert!(
        stdout.contains("Total Tags                    77"),
        "total tags"
    );
    assert!(
        stdout.contains("Total Assigned Tags           67"),
        "assigned tags"
    );
    assert!(stdout.contains("CDS_Exons"), "CDS_Exons row present");
    assert!(stdout.contains("5'UTR_Exons"), "5'UTR_Exons row present");
    assert!(stdout.contains("3'UTR_Exons"), "3'UTR_Exons row present");
    assert!(stdout.contains("Introns"), "Introns row present");
    assert!(stdout.contains("TSS_up_1kb"), "TSS_up_1kb row present");
    assert!(stdout.contains("TES_down_1kb"), "TES_down_1kb row present");
}

#[test]
fn smoke_golden_cds_count() {
    let bam = Path::new(GOLDEN).join("reads.bam");
    let bed = Path::new(GOLDEN).join("genes.bed12");
    let stdout = run_bin(bam.to_str().unwrap(), bed.to_str().unwrap());

    // CDS_Exons: 7600 bases, 38 tags
    assert!(
        stdout
            .lines()
            .any(|l| l.starts_with("CDS_Exons") && l.contains("7600") && l.contains("38")),
        "CDS_Exons bases=7600 tags=38\n{stdout}"
    );
}

#[test]
fn smoke_golden_intron_count() {
    let bam = Path::new(GOLDEN).join("reads.bam");
    let bed = Path::new(GOLDEN).join("genes.bed12");
    let stdout = run_bin(bam.to_str().unwrap(), bed.to_str().unwrap());

    // Introns: 4000 bases, 10 tags
    assert!(
        stdout
            .lines()
            .any(|l| l.starts_with("Introns") && l.contains("4000") && l.contains("10")),
        "Introns bases=4000 tags=10\n{stdout}"
    );
}

#[test]
fn smoke_json_output() {
    let bam = Path::new(GOLDEN).join("reads.bam");
    let bed = Path::new(GOLDEN).join("genes.bed12");
    let our_bin = env!("CARGO_BIN_EXE_rsomics-bam-read-dist");
    let out = Command::new(our_bin)
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-r",
            bed.to_str().unwrap(),
            "-t",
            "1",
            "--json",
        ])
        .output()
        .expect("failed to run rsomics-bam-read-dist");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    // The framework appends a status envelope on a second line after the tool's JSON.
    // Parse only the first JSON object.
    let first_line = stdout.lines().collect::<Vec<_>>()[..].join("\n");
    let v: serde_json::Value = serde_json::Deserializer::from_str(&first_line)
        .into_iter::<serde_json::Value>()
        .next()
        .expect("at least one JSON value")
        .expect("valid JSON");
    assert_eq!(v["total_reads"], 73);
    assert_eq!(v["total_tags"], 77);
    assert_eq!(v["cds_exons_tags"], 38);
}
