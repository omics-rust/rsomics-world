use std::path::PathBuf;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-multibam-summary"))
}

fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

fn tmp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("rsomics-multibam-summary-smoke");
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

#[test]
fn bins_header_and_columns() {
    let out = bin()
        .args(["-b", &golden("a.bam"), &golden("b.bam")])
        .args(["-o", "-"])
        .args(["--bin-size", "200"])
        .arg("--quiet")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(
        lines[0], "#'chr'\t'start'\t'end'\t'a.bam'\t'b.bam'",
        "header must be single-quoted, tab-separated"
    );
    for line in &lines[1..] {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            cols.len(),
            5,
            "row must be chr start end + 2 samples: {line}"
        );
        cols[1].parse::<u64>().unwrap();
        cols[2].parse::<u64>().unwrap();
        // Counts print as float64 (e.g. "4.0").
        assert!(
            cols[3].ends_with(".0"),
            "count must be float64: {}",
            cols[3]
        );
        cols[3].parse::<f64>().unwrap();
        cols[4].parse::<f64>().unwrap();
    }
}

#[test]
fn bed_mode_one_row_per_region() {
    let out = bin()
        .args(["-b", &golden("a.bam"), &golden("b.bam")])
        .args(["--bed", &golden("regions.bed")])
        .args(["-o", "-"])
        .arg("--quiet")
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let data: Vec<&str> = s.trim().lines().skip(1).collect();
    // regions.bed has 5 lines → 5 data rows, in declaration order.
    assert_eq!(data.len(), 5);
    assert!(data[0].starts_with("chr1\t0\t200\t"));
    assert!(data[4].starts_with("chr2\t400\t450\t"));
}

#[test]
fn writes_to_file() {
    let out_path = tmp("counts.tab");
    let status = bin()
        .args(["-b", &golden("a.bam"), &golden("b.bam")])
        .args(["-o", out_path.to_str().unwrap()])
        .args(["--bin-size", "500"])
        .arg("--quiet")
        .status()
        .unwrap();
    assert!(status.success());
    let s = std::fs::read_to_string(&out_path).unwrap();
    assert!(s.starts_with("#'chr'\t'start'\t'end'\t"));
}

#[test]
fn missing_bam_fails_loud() {
    let out = bin()
        .args(["-b", "/no/such/a.bam", "/no/such/b.bam"])
        .args(["-o", "-"])
        .output()
        .unwrap();
    assert!(!out.status.success(), "must fail on missing input");
}
