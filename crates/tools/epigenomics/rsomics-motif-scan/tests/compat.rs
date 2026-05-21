use std::process::Command;
fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-motif-scan"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn positions_are_correct() {
    // small.fa seq1 = ATCGATCGATCG, search for ATCG → positions 0,4,8
    let out = Command::new(ours())
        .arg(golden("small.fa"))
        .args(["-m", "ATCG"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    let seq1_hits: Vec<(u64, u64)> = s
        .lines()
        .filter(|l| l.starts_with("seq1"))
        .map(|l| {
            let parts: Vec<&str> = l.split('\t').collect();
            (parts[1].parse().unwrap(), parts[2].parse().unwrap())
        })
        .collect();
    assert!(seq1_hits.contains(&(0, 4)), "should find ATCG at 0-4");
    assert!(seq1_hits.contains(&(4, 8)), "should find ATCG at 4-8");
}

#[test]
fn iupac_n_matches_any() {
    // Search for NNNN should match everywhere there's 4+ consecutive bases
    let out = Command::new(ours())
        .arg(golden("small.fa"))
        .args(["-m", "NNNN"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(!s.is_empty(), "NNNN should match somewhere");
}
