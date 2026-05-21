use std::process::Command;
fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-seq-stats"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn stats_match_known_values() {
    // small.fa has 5 seqs: 12, 20, 16, 4, 32 = 84 total bp
    let out = Command::new(ours())
        .arg(golden("small.fa"))
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("sequences\t5"));
    assert!(s.contains("total_bp\t84"));
    assert!(s.contains("min_len\t4"));
    assert!(s.contains("max_len\t32"));
}
