use std::process::Command;
fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-tsv-select"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn select_by_name() {
    let out = bin()
        .arg(golden("data.tsv"))
        .args(["-c", "gene", "log2fc"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("gene\tlog2fc"));
    assert!(s.contains("BRCA1\t2.5"));
    assert!(!s.contains("pvalue"));
}

#[test]
fn select_by_index() {
    let out = bin()
        .arg(golden("data.tsv"))
        .args(["-c", "1", "3"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("gene\tlog2fc"));
}
