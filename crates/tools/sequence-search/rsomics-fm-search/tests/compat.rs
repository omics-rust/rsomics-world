use std::process::Command;
fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-fm-search"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn count_matches_grep() {
    // Count ATCG in small.fa seq1 (ATCGATCGATCG) → 3 occurrences
    let out = Command::new(ours())
        .arg(golden("small.fa"))
        .args(["-p", "ATCG", "-c"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    let seq1_count: u64 = s
        .lines()
        .find(|l| l.starts_with("seq1"))
        .and_then(|l| l.split('\t').nth(1)?.parse().ok())
        .unwrap_or(0);
    assert_eq!(seq1_count, 3, "ATCGATCGATCG has 3 occurrences of ATCG");
}

#[test]
fn no_match_returns_empty() {
    let out = Command::new(ours())
        .arg(golden("small.fa"))
        .args(["-p", "ZZZZZ", "-c"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.trim().is_empty(), "no match should produce no output");
}
