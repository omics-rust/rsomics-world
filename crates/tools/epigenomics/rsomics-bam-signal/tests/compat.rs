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
        .args(["--out-file-format", "bedgraph"])
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

/// Compare our bigWig output vs deeptools bigWig output value-by-value using
/// `multiBigwigSummary bins`.  Both bigWigs are read at 50 bp bins (matching
/// the write resolution) and each bin's value must match exactly.
#[test]
fn bigwig_values_match_deeptools() {
    if !have("bamCoverage") || !have("multiBigwigSummary") {
        eprintln!("skipping: bamCoverage or multiBigwigSummary not found");
        return;
    }

    let ver = deeptools_version().unwrap_or_default();
    let bam = golden("small.bam");

    let dir = std::env::temp_dir().join("rsomics-bam-signal-bw-compat");
    let _ = std::fs::create_dir_all(&dir);

    let ours_bw = dir.join("ours.bw");
    let dt_bw = dir.join("dt.bw");
    let summary_npz = dir.join("summary.npz");
    let summary_tab = dir.join("summary.tab");

    // Write our bigWig.
    let status = Command::new(ours())
        .arg(&bam)
        .args(["-o", ours_bw.to_str().unwrap()])
        .args(["--out-file-format", "bigwig"])
        .args(["--bin-size", "50"])
        .args(["-q"])
        .status()
        .expect("rsomics-bam-signal failed to launch");
    assert!(
        status.success(),
        "rsomics-bam-signal (bigwig) exited non-zero"
    );

    // Write deeptools bigWig.
    let status = Command::new("bamCoverage")
        .args(["-b", &bam])
        .args(["-o", dt_bw.to_str().unwrap()])
        .args(["--outFileFormat", "bigwig"])
        .args(["--binSize", "50"])
        .args(["-p", "1"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("bamCoverage failed to launch");
    assert!(status.success(), "bamCoverage (bigwig) exited non-zero");

    // Compare using multiBigwigSummary bins at 50 bp resolution.
    let status = Command::new("multiBigwigSummary")
        .args(["bins"])
        .args(["-b", ours_bw.to_str().unwrap(), dt_bw.to_str().unwrap()])
        .args(["--binSize", "50"])
        .args(["-o", summary_npz.to_str().unwrap()])
        .args(["--outRawCounts", summary_tab.to_str().unwrap()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("multiBigwigSummary failed to launch");
    assert!(
        status.success(),
        "multiBigwigSummary failed — ours.bw may be unreadable by deeptools"
    );

    // Parse the tab file and verify all values match.
    let content = std::fs::read_to_string(&summary_tab).expect("reading summary.tab");
    let mut mismatches = 0usize;
    let mut compared = 0usize;
    for line in content.lines().skip(1) {
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 5 {
            continue;
        }
        let v_ours: f64 = cols[3].parse().unwrap_or(f64::NAN);
        let v_dt: f64 = cols[4].parse().unwrap_or(f64::NAN);
        compared += 1;
        if (v_ours - v_dt).abs() > 0.001 {
            mismatches += 1;
            eprintln!(
                "mismatch at {}\t{}\t{}: ours={} dt={}",
                cols[0], cols[1], cols[2], v_ours, v_dt
            );
        }
    }
    assert!(
        mismatches == 0,
        "bigWig values differ from deeptools in {mismatches}/{compared} bins ({ver})"
    );
}
