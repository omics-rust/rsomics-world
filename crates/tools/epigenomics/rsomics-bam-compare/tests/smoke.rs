use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-compare"))
}

fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn basic_log2_output() {
    let out = bin()
        .args(["--bam1", &golden("treat.bam")])
        .args(["--bam2", &golden("ctrl.bam")])
        .args(["-o", "-"])
        .args(["--out-file-format", "bedgraph"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = s.trim().lines().collect();
    assert!(!lines.is_empty(), "no output lines");
    for line in &lines {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(cols.len(), 4, "each line must have 4 columns: {line}");
        cols[1].parse::<u64>().expect("start must be numeric");
        cols[2].parse::<u64>().expect("end must be numeric");
        cols[3].parse::<f64>().expect("value must be numeric");
    }
}

#[test]
fn all_operations_run() {
    for op in [
        "log2",
        "ratio",
        "reciprocal_ratio",
        "subtract",
        "add",
        "mean",
        "first",
        "second",
    ] {
        let out = bin()
            .args(["--bam1", &golden("treat.bam")])
            .args(["--bam2", &golden("ctrl.bam")])
            .args(["-o", "-"])
            .args(["--out-file-format", "bedgraph"])
            .args(["--operation", op])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "operation {op} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            !String::from_utf8_lossy(&out.stdout).trim().is_empty(),
            "operation {op} produced no output"
        );
    }
}

#[test]
fn unknown_operation_fails() {
    let out = bin()
        .args(["--bam1", &golden("treat.bam")])
        .args(["--bam2", &golden("ctrl.bam")])
        .args(["-o", "-"])
        .args(["--out-file-format", "bedgraph"])
        .args(["--operation", "bogus"])
        .output()
        .unwrap();
    assert!(!out.status.success(), "unknown operation must fail");
}

#[test]
fn two_value_pseudocount_runs() {
    let out = bin()
        .args(["--bam1", &golden("treat.bam")])
        .args(["--bam2", &golden("ctrl.bam")])
        .args(["-o", "-"])
        .args(["--out-file-format", "bedgraph"])
        .args(["--pseudocount", "2", "3"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "two-value pseudocount failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn bigwig_stdout_rejected() {
    let out = bin()
        .args(["--bam1", &golden("treat.bam")])
        .args(["--bam2", &golden("ctrl.bam")])
        .args(["-o", "-"])
        .args(["--out-file-format", "bigwig"])
        .output()
        .unwrap();
    assert!(!out.status.success(), "bigwig stdout must be rejected");
}

#[test]
fn bigwig_writes_file() {
    let dir = tempfile::tempdir().unwrap();
    let bw = dir.path().join("out.bw");
    let out = bin()
        .args(["--bam1", &golden("treat.bam")])
        .args(["--bam2", &golden("ctrl.bam")])
        .args(["-o", bw.to_str().unwrap()])
        .args(["--out-file-format", "bigwig"])
        .args(["-q"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "bigwig write failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // bigWig magic bytes: 0x888FFC26
    let bytes = std::fs::read(&bw).unwrap();
    assert!(bytes.len() >= 4, "bigwig file too small");
    assert_eq!(
        &bytes[..4],
        &[0x26, 0xFC, 0x8F, 0x88],
        "missing bigWig magic"
    );
}

#[test]
fn bedgraph_format_explicit() {
    let out = bin()
        .args(["--bam1", &golden("treat.bam")])
        .args(["--bam2", &golden("ctrl.bam")])
        .args(["-o", "-"])
        .args(["--out-file-format", "bedgraph"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "explicit bedgraph failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(!s.trim().is_empty(), "no bedgraph output");
}
