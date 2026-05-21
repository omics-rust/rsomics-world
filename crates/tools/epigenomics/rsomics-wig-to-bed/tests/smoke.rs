use std::process::Command;
fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-wig-to-bed"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn threshold_filter() {
    let out = bin()
        .arg(golden("signal.wig"))
        .args(["--threshold", "3.0"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 2); // 5.0 and 8.0 pass threshold
}
