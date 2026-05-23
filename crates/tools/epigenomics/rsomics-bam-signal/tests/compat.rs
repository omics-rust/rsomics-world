use std::process::{Command, Stdio};

fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-bam-signal"))
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

fn deeptools_version() -> Option<String> {
    let out = Command::new("bamCoverage").arg("--version").output().ok()?;
    Some(String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

/// Run bamCoverage on the golden BAM and return its bedGraph output.
fn run_deeptools(bam: &str, bin_size: u32) -> String {
    let dir = std::env::temp_dir().join("rsomics-bam-signal-compat");
    let _ = std::fs::create_dir_all(&dir);
    let out = dir.join(format!("deeptools_{bin_size}.bedgraph"));

    let status = Command::new("bamCoverage")
        .args(["-b", bam])
        .args(["-o", out.to_str().unwrap()])
        .args(["--outFileFormat", "bedgraph"])
        .args(["--binSize", &bin_size.to_string()])
        .stdout(Stdio::null())
        .status()
        .expect("bamCoverage failed to launch");
    assert!(status.success(), "bamCoverage exited non-zero");

    std::fs::read_to_string(&out).expect("reading bamCoverage output")
}

/// Run ours on the golden BAM and return its bedGraph output.
fn run_ours(bam: &str, bin_size: u32) -> String {
    let out = Command::new(ours())
        .arg(bam)
        .args(["-o", "-"])
        .args(["--bin-size", &bin_size.to_string()])
        .output()
        .expect("rsomics-bam-signal failed to launch");
    assert!(
        out.status.success(),
        "rsomics-bam-signal failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn bedgraph_matches_deeptools_binsize50() {
    if !have("bamCoverage") {
        eprintln!("skipping: bamCoverage not found (install deeptools)");
        return;
    }

    let ver = deeptools_version().unwrap_or_default();
    eprintln!("deeptools version: {ver}");

    let bam = golden("small.bam");
    let ours_out = run_ours(&bam, 50);
    let dt_out = run_deeptools(&bam, 50);

    assert_eq!(
        ours_out.trim(),
        dt_out.trim(),
        "bedGraph output mismatch vs deeptools {ver} at binSize=50"
    );
}

#[test]
fn bedgraph_matches_deeptools_binsize100() {
    if !have("bamCoverage") {
        eprintln!("skipping: bamCoverage not found (install deeptools)");
        return;
    }

    let bam = golden("small.bam");
    let ours_out = run_ours(&bam, 100);
    let dt_out = run_deeptools(&bam, 100);

    assert_eq!(
        ours_out.trim(),
        dt_out.trim(),
        "bedGraph output mismatch vs deeptools at binSize=100"
    );
}
