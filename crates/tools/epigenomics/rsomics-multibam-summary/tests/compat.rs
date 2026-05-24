//! Byte-exact compatibility against deeptools `multiBamSummary --outRawCounts`,
//! for both `bins` and `BED-file` modes, on the golden BAM pair.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn ours() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_rsomics-multibam-summary"))
}

fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

fn have(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn deeptools_version() -> String {
    Command::new("multiBamSummary")
        .arg("--version")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .unwrap_or_default()
}

fn tmp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("rsomics-multibam-summary-compat");
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

fn run_deeptools_bins(a: &str, b: &str, bin_size: u32) -> String {
    let raw = tmp("dt_bins.tab");
    let npz = tmp("dt_bins.npz");
    let status = Command::new("multiBamSummary")
        .arg("bins")
        .args(["-b", a, b])
        .args(["-o", npz.to_str().unwrap()])
        .args(["--outRawCounts", raw.to_str().unwrap()])
        .args(["--binSize", &bin_size.to_string()])
        .args(["-p", "1"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("multiBamSummary failed to launch");
    assert!(status.success(), "multiBamSummary bins exited non-zero");
    std::fs::read_to_string(&raw).expect("reading deeptools outRawCounts")
}

fn run_deeptools_bed(a: &str, b: &str, bed: &str) -> String {
    let raw = tmp("dt_bed.tab");
    let npz = tmp("dt_bed.npz");
    let status = Command::new("multiBamSummary")
        .arg("BED-file")
        .args(["--BED", bed])
        .args(["-b", a, b])
        .args(["-o", npz.to_str().unwrap()])
        .args(["--outRawCounts", raw.to_str().unwrap()])
        .args(["-p", "1"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("multiBamSummary failed to launch");
    assert!(status.success(), "multiBamSummary BED-file exited non-zero");
    std::fs::read_to_string(&raw).expect("reading deeptools outRawCounts")
}

fn run_ours_bins(a: &str, b: &str, bin_size: u32) -> String {
    let out = Command::new(ours())
        .args(["-b", a, b])
        .args(["-o", "-"])
        .args(["--bin-size", &bin_size.to_string()])
        .arg("--quiet")
        .output()
        .expect("rsomics-multibam-summary failed to launch");
    assert!(
        out.status.success(),
        "ours failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn run_ours_bed(a: &str, b: &str, bed: &str) -> String {
    let out = Command::new(ours())
        .args(["-b", a, b])
        .args(["--bed", bed])
        .args(["-o", "-"])
        .arg("--quiet")
        .output()
        .expect("rsomics-multibam-summary failed to launch");
    assert!(
        out.status.success(),
        "ours failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn bins_matches_deeptools() {
    if !have("multiBamSummary") {
        eprintln!("skipping: multiBamSummary not found (install deeptools)");
        return;
    }
    let ver = deeptools_version();
    eprintln!("deeptools version: {ver}");

    let a = golden("a.bam");
    let b = golden("b.bam");
    // deeptools rejects bins wide enough to yield < ~3 sampling bins on this
    // small golden genome (its `numberOfSamples` floor), so the default-10kb
    // case can't be exercised here; the bin-layout maths is covered at 200/250.
    for bin_size in [200u32, 250] {
        let ours = run_ours_bins(&a, &b, bin_size);
        let dt = run_deeptools_bins(&a, &b, bin_size);
        assert_eq!(
            ours, dt,
            "bins --outRawCounts mismatch vs deeptools {ver} at binSize={bin_size}"
        );
    }
}

#[test]
fn bed_file_matches_deeptools() {
    if !have("multiBamSummary") {
        eprintln!("skipping: multiBamSummary not found (install deeptools)");
        return;
    }
    let a = golden("a.bam");
    let b = golden("b.bam");
    // Both a pre-sorted BED and one in shuffled declaration order: deeptools
    // emits rows in (chrom-header, ascending-position) order regardless, so the
    // two must produce identical output and both must match deeptools.
    for bed_name in ["regions.bed", "regions_unsorted.bed"] {
        let bed = golden(bed_name);
        let ours = run_ours_bed(&a, &b, &bed);
        let dt = run_deeptools_bed(&a, &b, &bed);
        assert_eq!(
            ours,
            dt,
            "BED-file --outRawCounts mismatch vs deeptools {} on {bed_name}",
            deeptools_version()
        );
    }
}
