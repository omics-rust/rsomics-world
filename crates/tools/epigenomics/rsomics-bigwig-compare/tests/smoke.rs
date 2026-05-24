use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bigwig-compare"))
}

fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn log2_runs_to_stdout() {
    let out = bin()
        .args(["--bigwig1", &golden("a.bw"), "--bigwig2", &golden("b.bw")])
        .args(["-o", "-", "--operation", "log2", "--bin-size", "50", "-q"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(!text.is_empty(), "empty output");
    for line in text.lines() {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(cols.len(), 4, "bedGraph line must have 4 columns: {line}");
        cols[1].parse::<u64>().unwrap();
        cols[2].parse::<u64>().unwrap();
    }
}

#[test]
fn writes_output_file() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let out = bin()
        .args(["--bigwig1", &golden("a.bw"), "--bigwig2", &golden("b.bw")])
        .args(["-o", tmp.path().to_str().unwrap(), "-q"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let text = std::fs::read_to_string(tmp.path()).unwrap();
    assert!(text.contains("chr1"), "expected chr1 in output");
}

#[test]
fn fixed_step_emits_all_bins() {
    let merged = bin()
        .args(["--bigwig1", &golden("a.bw"), "--bigwig2", &golden("b.bw")])
        .args(["-o", "-", "--bin-size", "50", "-q"])
        .output()
        .unwrap();
    let fixed = bin()
        .args(["--bigwig1", &golden("a.bw"), "--bigwig2", &golden("b.bw")])
        .args(["-o", "-", "--bin-size", "50", "--fixed-step", "-q"])
        .output()
        .unwrap();
    let n_merged = String::from_utf8(merged.stdout).unwrap().lines().count();
    let n_fixed = String::from_utf8(fixed.stdout).unwrap().lines().count();
    assert!(
        n_fixed >= n_merged,
        "fixed-step ({n_fixed}) should emit at least as many lines as merged ({n_merged})"
    );
}

#[test]
fn unsupported_format_fails() {
    let out = bin()
        .args(["--bigwig1", &golden("a.bw"), "--bigwig2", &golden("b.bw")])
        .args(["-o", "-", "--out-file-format", "bigwig", "-q"])
        .output()
        .unwrap();
    assert!(!out.status.success(), "bigwig output must be rejected");
}

#[test]
fn missing_bigwig_fails() {
    let out = bin()
        .args(["--bigwig1", "/nonexistent.bw", "--bigwig2", &golden("b.bw")])
        .args(["-o", "-", "-q"])
        .output()
        .unwrap();
    assert!(!out.status.success(), "missing input must fail loudly");
}
