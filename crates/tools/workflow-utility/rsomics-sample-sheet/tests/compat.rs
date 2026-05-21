use std::process::Command;
fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-sample-sheet"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn output_has_header() {
    let out = Command::new(ours())
        .arg(golden("sheet.tsv"))
        .output()
        .unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    let header = s.lines().next().unwrap_or("");
    assert!(
        header.contains("sample_id"),
        "output should have sample_id header"
    );
    assert!(
        header.contains("status"),
        "output should have status column"
    );
}
