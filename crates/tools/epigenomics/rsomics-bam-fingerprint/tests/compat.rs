use std::process::{Command, Stdio};

fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-bam-fingerprint"))
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
    Command::new("plotFingerprint")
        .arg("--version")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .unwrap_or_default()
}

/// Run `plotFingerprint --outRawCounts` single-threaded; return the per-bin
/// count rows (skipping the two-line `#`/label header).
fn run_deeptools(bam: &str, bin_size: u32, n_samples: u64) -> Vec<String> {
    let dir = std::env::temp_dir().join("rsomics-bam-fingerprint-compat");
    std::fs::create_dir_all(&dir).unwrap();
    let out = dir.join(format!("dt_{bin_size}_{n_samples}.tab"));

    let status = Command::new("plotFingerprint")
        .args(["-b", bam])
        .args(["--binSize", &bin_size.to_string()])
        .args(["--numberOfSamples", &n_samples.to_string()])
        .args(["-p", "1"])
        .args(["--outRawCounts", out.to_str().unwrap()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("plotFingerprint failed to launch");
    assert!(status.success(), "plotFingerprint exited non-zero");

    data_rows(&std::fs::read_to_string(&out).expect("reading plotFingerprint output"))
}

fn run_ours(bam: &str, bin_size: u32, n_samples: u64) -> Vec<String> {
    let out = Command::new(ours())
        .args(["-b", bam])
        .args(["--bin-size", &bin_size.to_string()])
        .args(["--number-of-samples", &n_samples.to_string()])
        .args(["--out-raw-counts", "-"])
        .output()
        .expect("rsomics-bam-fingerprint failed to launch");
    assert!(
        out.status.success(),
        "rsomics-bam-fingerprint failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    data_rows(&String::from_utf8(out.stdout).unwrap())
}

/// Drop the `#plotFingerprint --outRawCounts` line and the quoted-label line;
/// keep the integer per-bin rows.
fn data_rows(s: &str) -> Vec<String> {
    s.lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with('\''))
        .map(str::to_owned)
        .collect()
}

fn assert_matches(bam: &str, bin_size: u32, n_samples: u64) {
    let dt = run_deeptools(bam, bin_size, n_samples);
    let ours = run_ours(bam, bin_size, n_samples);
    assert_eq!(
        ours,
        dt,
        "per-bin counts mismatch vs deeptools {} at binSize={bin_size} numberOfSamples={n_samples}",
        deeptools_version()
    );
}

#[test]
fn raw_counts_match_sparse_path() {
    if !have("plotFingerprint") {
        eprintln!("skipping: plotFingerprint not found (install deeptools)");
        return;
    }
    eprintln!("deeptools version: {}", deeptools_version());
    // stepSize != binSize → single-bin (sparse) coverage path.
    assert_matches(&golden("small.bam"), 100, 50);
    assert_matches(&golden("small.bam"), 150, 40);
    assert_matches(&golden("multi.bam"), 100, 150);
}

#[test]
fn raw_counts_match_tiled_path() {
    if !have("plotFingerprint") {
        eprintln!("skipping: plotFingerprint not found (install deeptools)");
        return;
    }
    // stepSize == binSize → tiled-chunk coverage path (the deeptools spread
    // arithmetic, including its wide-read tile over-count quirk).
    assert_matches(&golden("small.bam"), 200, 50);
    assert_matches(&golden("multi.bam"), 200, 60);
}

#[test]
fn raw_counts_match_cigar_blocks() {
    if !have("plotFingerprint") {
        eprintln!("skipping: plotFingerprint not found (install deeptools)");
        return;
    }
    // Reads with N (splice), D (deletion), I (insertion), S (soft-clip):
    // get_blocks() splitting must match pysam.
    assert_matches(&golden("cigar.bam"), 50, 80);
    assert_matches(&golden("cigar.bam"), 30, 100);
    assert_matches(&golden("cigar.bam"), 100, 40);
}

#[test]
fn raw_counts_match_multichunk() {
    if !have("plotFingerprint") {
        eprintln!("skipping: plotFingerprint not found (install deeptools)");
        return;
    }
    // Dense genome forces chunkSize < chromosome length → multi-chunk layout
    // with chunk-boundary bins dropped.
    assert_matches(&golden("dense.bam"), 50, 100_000);
    assert_matches(&golden("dense.bam"), 100, 100_000);
}
