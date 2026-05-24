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
        .args(["--pseudocount", "2", "3"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "two-value pseudocount failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
