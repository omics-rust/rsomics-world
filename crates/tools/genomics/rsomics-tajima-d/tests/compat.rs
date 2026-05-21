use std::process::Command;
fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-tajima-d"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn d_is_finite() {
    let out = Command::new(ours())
        .arg(golden("sfs.tsv"))
        .args(["-n", "20"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let d: f64 = String::from_utf8(out.stdout)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert!(d.is_finite(), "D must be finite");
    assert!(
        (-4.0..4.0).contains(&d),
        "D should be in reasonable range, got {d}"
    );
}
