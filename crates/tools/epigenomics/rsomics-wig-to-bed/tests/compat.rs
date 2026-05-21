use std::process::Command;
fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-wig-to-bed"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn all_values_above_threshold() {
    let out = Command::new(ours())
        .arg(golden("signal.wig"))
        .args(["--threshold", "3.0"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    for line in s.lines() {
        let val: f64 = line.split('\t').nth(3).unwrap().parse().unwrap();
        assert!(val >= 3.0, "value {val} below threshold 3.0");
    }
}

#[test]
fn threshold_0_keeps_all() {
    let out = Command::new(ours())
        .arg(golden("signal.wig"))
        .args(["--threshold", "0.0"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert_eq!(
        s.lines().count(),
        4,
        "threshold=0 should keep all 4 windows"
    );
}
