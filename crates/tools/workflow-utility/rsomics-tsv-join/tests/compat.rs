use std::process::Command;
fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-tsv-join"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn inner_join_only_shared_keys() {
    let out = Command::new(ours())
        .arg(golden("left.tsv"))
        .arg(golden("right.tsv"))
        .args(["-k", "gene"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    let data: Vec<&str> = s.lines().skip(1).collect();
    for line in &data {
        let gene = line.split('\t').next().unwrap();
        assert!(
            gene == "BRCA1" || gene == "EGFR",
            "unexpected gene in inner join: {gene}"
        );
    }
    assert!(!s.contains("TP53"), "TP53 only in left, should not appear");
    assert!(!s.contains("MYC"), "MYC only in right, should not appear");
}

#[test]
fn joined_row_has_columns_from_both() {
    let out = Command::new(ours())
        .arg(golden("left.tsv"))
        .arg(golden("right.tsv"))
        .args(["-k", "gene"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    let header = s.lines().next().unwrap();
    assert!(header.contains("expr"), "should have left column 'expr'");
    assert!(header.contains("chrom"), "should have right column 'chrom'");
}
