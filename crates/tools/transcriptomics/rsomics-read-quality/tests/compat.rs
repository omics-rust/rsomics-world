/// Compatibility test against RSeQC read_quality.py 2.6.2.
///
/// Requires `read_quality.py` in PATH (conda env `rs-rseqc`).
/// Skipped automatically when the upstream binary is absent.
///
/// Byte-exact comparison of the data lines in `.qual.r`, excluding the
/// `pdf(...)` and `dev.off()` lines that embed absolute paths.
use std::fs;
use std::path::Path;
use std::process::Command;

fn upstream_available() -> bool {
    // Check the conda env directly since read_quality.py may not be on PATH.
    Command::new("conda")
        .args(["run", "-n", "rs-rseqc", "read_quality.py", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn binary_path() -> std::path::PathBuf {
    // cargo test builds into the same target dir as the binary.
    let mut p = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    // Walk up from deps/ to the profile dir.
    if p.ends_with("deps") {
        p.pop();
    }
    p.join("rsomics-read-quality")
}

fn strip_path_lines(content: &str) -> String {
    content
        .lines()
        .filter(|l| !l.starts_with("pdf(") && !l.starts_with("dev.off()"))
        .filter(|l| !l.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn run_upstream(input: &Path, prefix: &str, mapq: u8) {
    let status = Command::new("conda")
        .args([
            "run",
            "-n",
            "rs-rseqc",
            "read_quality.py",
            "-i",
            input.to_str().unwrap(),
            "-o",
            prefix,
            "-q",
            &mapq.to_string(),
        ])
        .status()
        .expect("conda run failed");
    assert!(status.success(), "read_quality.py failed with {status}");
}

fn run_ours(input: &Path, prefix: &str, mapq: u8) {
    let bin = binary_path();
    let status = Command::new(&bin)
        .args([
            "-i",
            input.to_str().unwrap(),
            "-o",
            prefix,
            "--mapq",
            &mapq.to_string(),
        ])
        .status()
        .unwrap_or_else(|e| panic!("failed to run {}: {e}", bin.display()));
    assert!(
        status.success(),
        "rsomics-read-quality failed with {status}"
    );
}

#[test]
fn compat_default_mapq() {
    if !upstream_available() {
        eprintln!("SKIP: read_quality.py not found (conda env rs-rseqc absent)");
        return;
    }

    let golden = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/test.bam");
    let dir = tempfile::tempdir().unwrap();
    let upstream_prefix = dir.path().join("upstream").to_str().unwrap().to_owned();
    let ours_prefix = dir.path().join("ours").to_str().unwrap().to_owned();

    run_upstream(&golden, &upstream_prefix, 30);
    run_ours(&golden, &ours_prefix, 30);

    let upstream_r = fs::read_to_string(format!("{upstream_prefix}.qual.r")).unwrap();
    let ours_r = fs::read_to_string(format!("{ours_prefix}.qual.r")).unwrap();

    let upstream_data = strip_path_lines(&upstream_r);
    let ours_data = strip_path_lines(&ours_r);

    assert_eq!(
        upstream_data, ours_data,
        "qual.r data mismatch vs RSeQC 2.6.2\n\
         --- upstream ---\n{upstream_data}\n\
         --- ours ---\n{ours_data}"
    );
}

#[test]
fn compat_low_mapq() {
    if !upstream_available() {
        eprintln!("SKIP: read_quality.py not found (conda env rs-rseqc absent)");
        return;
    }

    let golden = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/test.bam");
    let dir = tempfile::tempdir().unwrap();
    let upstream_prefix = dir.path().join("upstream").to_str().unwrap().to_owned();
    let ours_prefix = dir.path().join("ours").to_str().unwrap().to_owned();

    run_upstream(&golden, &upstream_prefix, 0);
    run_ours(&golden, &ours_prefix, 0);

    let upstream_r = fs::read_to_string(format!("{upstream_prefix}.qual.r")).unwrap();
    let ours_r = fs::read_to_string(format!("{ours_prefix}.qual.r")).unwrap();

    assert_eq!(
        strip_path_lines(&upstream_r),
        strip_path_lines(&ours_r),
        "compat_low_mapq mismatch"
    );
}
