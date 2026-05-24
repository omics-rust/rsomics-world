use std::process::{Command, Stdio};
use std::sync::Mutex;

// Concurrent `bamCompare` invocations corrupt each other's readCount sampling
// (a deeptools-side shared-temp artifact), so the upstream calls are serialised.
// Ours is deterministic; this only constrains the reference side.
static DEEPTOOLS_LOCK: Mutex<()> = Mutex::new(());

fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-bam-compare"))
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
    let out = Command::new("bamCompare").arg("--version").output().ok()?;
    Some(String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

/// Run bamCompare on the golden BAMs and return its bedGraph output.
fn run_deeptools(op: &str, bin_size: u32, pseudocount: &[&str]) -> String {
    let dir = std::env::temp_dir().join("rsomics-bam-compare-compat");
    let _ = std::fs::create_dir_all(&dir);
    let out = dir.join(format!("dt_{op}_{bin_size}.bedgraph"));

    let mut cmd = Command::new("bamCompare");
    cmd.args(["-b1", &golden("treat.bam")])
        .args(["-b2", &golden("ctrl.bam")])
        .args(["-o", out.to_str().unwrap()])
        .args(["--outFileFormat", "bedgraph"])
        .args(["--binSize", &bin_size.to_string()])
        .args(["--operation", op])
        .args(["-p", "1"]);
    if !pseudocount.is_empty() {
        cmd.arg("--pseudocount").args(pseudocount);
    }
    let status = cmd
        .stdout(Stdio::null())
        .status()
        .expect("bamCompare failed to launch");
    assert!(status.success(), "bamCompare exited non-zero");

    std::fs::read_to_string(&out).expect("reading bamCompare output")
}

/// Run ours on the golden BAMs and return its bedGraph output.
fn run_ours(op: &str, bin_size: u32, pseudocount: &[&str]) -> String {
    let mut cmd = Command::new(ours());
    cmd.args(["--bam1", &golden("treat.bam")])
        .args(["--bam2", &golden("ctrl.bam")])
        .args(["-o", "-"])
        .args(["--bin-size", &bin_size.to_string()])
        .args(["--operation", op]);
    if !pseudocount.is_empty() {
        cmd.arg("--pseudocount").args(pseudocount);
    }
    let out = cmd.output().expect("rsomics-bam-compare failed to launch");
    assert!(
        out.status.success(),
        "rsomics-bam-compare failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn assert_matches(op: &str, bin_size: u32, pseudocount: &[&str]) {
    if !have("bamCompare") {
        eprintln!("skipping: bamCompare not found (install deeptools)");
        return;
    }
    let ver = deeptools_version().unwrap_or_default();
    let ours_out = run_ours(op, bin_size, pseudocount);
    let dt_out = {
        let _guard = DEEPTOOLS_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        run_deeptools(op, bin_size, pseudocount)
    };
    assert_eq!(
        ours_out.trim(),
        dt_out.trim(),
        "bedGraph mismatch vs deeptools {ver}: op={op} bin={bin_size} pc={pseudocount:?}"
    );
}

#[test]
fn log2_binsize50() {
    assert_matches("log2", 50, &[]);
}

#[test]
fn log2_binsize100() {
    assert_matches("log2", 100, &[]);
}

#[test]
fn ratio_binsize50() {
    assert_matches("ratio", 50, &[]);
}

#[test]
fn reciprocal_ratio_binsize50() {
    assert_matches("reciprocal_ratio", 50, &[]);
}

#[test]
fn subtract_binsize50() {
    assert_matches("subtract", 50, &[]);
}

#[test]
fn add_binsize50() {
    assert_matches("add", 50, &[]);
}

#[test]
fn mean_binsize50() {
    assert_matches("mean", 50, &[]);
}

#[test]
fn first_second_binsize50() {
    assert_matches("first", 50, &[]);
    assert_matches("second", 50, &[]);
}

#[test]
fn log2_custom_pseudocount() {
    assert_matches("log2", 50, &["2", "3"]);
}

#[test]
fn ratio_single_pseudocount() {
    assert_matches("ratio", 50, &["5"]);
}
