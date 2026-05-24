use std::path::PathBuf;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-multibigwig-summary"))
}

fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

fn tmp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("rsomics-multibigwig-summary-smoke");
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

#[test]
fn bins_header_and_columns() {
    let out = bin()
        .args(["-b", &golden("a.bw"), &golden("b.bw")])
        .args(["-o", "-"])
        .args(["--bin-size", "100"])
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
        lines[0], "#'chr'\t'start'\t'end'\t'a.bw'\t'b.bw'",
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
        // Values print as Python float (e.g. "2.0", "nan").
        assert!(
            cols[3].ends_with(".0") || cols[3].contains('.') || cols[3] == "nan",
            "value must be float-like: {}",
            cols[3]
        );
    }
}

#[test]
fn bins_row_count() {
    let out = bin()
        .args(["-b", &golden("a.bw"), &golden("b.bw")])
        .args(["-o", "-"])
        .args(["--bin-size", "100"])
        .arg("--quiet")
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    // chr1=500bp → 5 bins; chr2=300bp → 3 bins = 8 rows
    let data: Vec<&str> = s.trim().lines().skip(1).collect();
    assert_eq!(data.len(), 8, "expected 8 bins (5 chr1 + 3 chr2)");
}

#[test]
fn bed_mode_rows_sorted() {
    let out = bin()
        .args(["-b", &golden("a.bw"), &golden("b.bw")])
        .args(["--bed", &golden("regions_unsorted.bed")])
        .args(["-o", "-"])
        .arg("--quiet")
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let data: Vec<&str> = s.trim().lines().skip(1).collect();
    assert_eq!(data.len(), 4, "4 BED regions → 4 rows");
    // First row must be chr1, regardless of input BED order.
    assert!(data[0].starts_with("chr1\t"), "first row must be chr1");
    assert!(data[2].starts_with("chr2\t"), "third row must be chr2");
}

#[test]
fn writes_to_file() {
    let out_path = tmp("counts.tab");
    let status = bin()
        .args(["-b", &golden("a.bw"), &golden("b.bw")])
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
fn missing_bw_fails_loud() {
    let out = bin()
        .args(["-b", "/no/such/a.bw", "/no/such/b.bw"])
        .args(["-o", "-"])
        .output()
        .unwrap();
    assert!(!out.status.success(), "must fail on missing input");
}
