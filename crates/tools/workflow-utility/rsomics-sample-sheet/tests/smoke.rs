use std::process::Command;
fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-sample-sheet"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn validate_sheet() {
    // File paths don't exist, so validation should fail
    let out = bin().arg(golden("sheet.tsv")).output().unwrap();
    assert!(!out.status.success()); // expected to fail — paths don't exist
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("invalid") || err.contains("failed"));
}
