use std::process::Command;
fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-tax-assign"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn classify_empty_db() {
    let out = bin()
        .arg(golden("reads.fq"))
        .args(["-d", &golden("empty_db.tsv"), "-k", "5"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    // All reads should be unclassified (U) with empty DB
    let u_count = s.lines().filter(|l| l.starts_with('U')).count();
    assert_eq!(u_count, 4);
}
