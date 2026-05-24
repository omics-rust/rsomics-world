//! Byte-exact compatibility against deeptools `multiBigwigSummary --outRawCounts`,
//! for both `bins` and `BED-file` modes, on the golden bigWig pair.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn ours() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_rsomics-multibigwig-summary"))
}

fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

fn fixture(n: &str) -> String {
    format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), n)
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
    Command::new("multiBigwigSummary")
        .arg("--version")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .unwrap_or_default()
}

fn tmp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("rsomics-multibigwig-summary-compat");
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

fn run_deeptools_bins(a: &str, b: &str, bin_size: u32) -> String {
    let raw = tmp("dt_bins.tab");
    let npz = tmp("dt_bins.npz");
    let status = Command::new("multiBigwigSummary")
        .arg("bins")
        .args(["-b", a, b])
        .args(["-o", npz.to_str().unwrap()])
        .args(["--outRawCounts", raw.to_str().unwrap()])
        .args(["--binSize", &bin_size.to_string()])
        .args(["-p", "1"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("multiBigwigSummary failed to launch");
    assert!(status.success(), "multiBigwigSummary bins exited non-zero");
    std::fs::read_to_string(&raw).expect("reading deeptools outRawCounts")
}

fn run_deeptools_bed(a: &str, b: &str, bed: &str) -> String {
    let raw = tmp("dt_bed.tab");
    let npz = tmp("dt_bed.npz");
    let status = Command::new("multiBigwigSummary")
        .arg("BED-file")
        .args(["--BED", bed])
        .args(["-b", a, b])
        .args(["-o", npz.to_str().unwrap()])
        .args(["--outRawCounts", raw.to_str().unwrap()])
        .args(["-p", "1"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("multiBigwigSummary failed to launch");
    assert!(
        status.success(),
        "multiBigwigSummary BED-file exited non-zero"
    );
    std::fs::read_to_string(&raw).expect("reading deeptools outRawCounts")
}

fn run_ours_bins(a: &str, b: &str, bin_size: u32) -> String {
    let out = Command::new(ours())
        .args(["-b", a, b])
        .args(["-o", "-"])
        .args(["--bin-size", &bin_size.to_string()])
        .arg("--quiet")
        .output()
        .expect("rsomics-multibigwig-summary failed to launch");
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
        .expect("rsomics-multibigwig-summary failed to launch");
    assert!(
        out.status.success(),
        "ours failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn bins_matches_deeptools() {
    if !have("multiBigwigSummary") {
        eprintln!("skipping: multiBigwigSummary not found (install deeptools)");
        return;
    }
    let ver = deeptools_version();
    eprintln!("deeptools version: {ver}");

    let a = golden("a.bw");
    let b = golden("b.bw");
    // bin sizes that produce several bins on the 500/300bp golden chromosomes.
    // These fall back to the full-resolution exact path (zoom level 500 > basesPerBin/2).
    for bin_size in [100u32, 150, 250] {
        let ours = run_ours_bins(&a, &b, bin_size);
        let dt = run_deeptools_bins(&a, &b, bin_size);
        assert_eq!(
            ours, dt,
            "bins --outRawCounts mismatch vs deeptools {ver} at binSize={bin_size}"
        );
    }
}

#[test]
fn bins_zoom_path_matches_deeptools() {
    // Exercises the zoom-approximate path: perf fixtures have zoom levels at
    // reduction 1008/4032/16128 etc.; binSize 10000 selects the 4032-level.
    // Bins with no-data positions diverge from the exact path — this is the
    // core correctness check for the zoom algorithm.
    if !have("multiBigwigSummary") {
        eprintln!("skipping: multiBigwigSummary not found (install deeptools)");
        return;
    }
    let ver = deeptools_version();
    eprintln!("deeptools version: {ver}");

    let a = fixture("perf_a.bw");
    let b = fixture("perf_b.bw");
    for bin_size in [10_000u32, 5_000, 20_000] {
        let ours = run_ours_bins(&a, &b, bin_size);
        let dt = run_deeptools_bins(&a, &b, bin_size);
        assert_eq!(
            ours, dt,
            "zoom-path bins mismatch vs deeptools {ver} at binSize={bin_size}"
        );
    }
}

#[test]
fn bed_file_matches_deeptools() {
    if !have("multiBigwigSummary") {
        eprintln!("skipping: multiBigwigSummary not found (install deeptools)");
        return;
    }
    let ver = deeptools_version();
    let a = golden("a.bw");
    let b = golden("b.bw");
    // Both declaration-ordered and shuffled BED must produce identical output
    // (deeptools sorts by chrom-order then start).
    for bed_name in ["regions.bed", "regions_unsorted.bed"] {
        let bed = golden(bed_name);
        let ours = run_ours_bed(&a, &b, &bed);
        let dt = run_deeptools_bed(&a, &b, &bed);
        assert_eq!(
            ours, dt,
            "BED-file --outRawCounts mismatch vs deeptools {ver} on {bed_name}"
        );
    }
}
