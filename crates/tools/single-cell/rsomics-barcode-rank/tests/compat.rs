use std::process::Command;
fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-barcode-rank"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn output_is_descending() {
    let out = Command::new(ours())
        .arg(golden("counts.tsv"))
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    let counts: Vec<u64> = s
        .lines()
        .skip(1)
        .filter_map(|l| l.split('\t').nth(1)?.parse().ok())
        .collect();
    for w in counts.windows(2) {
        assert!(w[0] >= w[1], "not descending: {} < {}", w[0], w[1]);
    }
}

#[test]
fn knee_is_reported() {
    let out = Command::new(ours())
        .arg(golden("counts.tsv"))
        .output()
        .unwrap();
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(err.contains("knee at rank"), "should report knee: {err}");
}
