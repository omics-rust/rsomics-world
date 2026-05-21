use std::process::Command;
fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-tajima-d"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn compute_d() {
    let out = bin()
        .arg(golden("sfs.tsv"))
        .args(["-n", "20"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    let d: f64 = s.trim().parse().unwrap();
    assert!(d.is_finite());
}
