use std::process::Command;
fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-deseq-prep"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn kept_genes_meet_threshold() {
    let out = Command::new(ours())
        .arg(golden("counts.tsv"))
        .args(["--min-count", "10", "--min-samples", "2"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    for line in s.lines().skip(1) {
        let counts: Vec<u64> = line
            .split('\t')
            .skip(1)
            .filter_map(|v| v.parse().ok())
            .collect();
        let above = counts.iter().filter(|&&c| c >= 10).count();
        assert!(above >= 2, "gene should have ≥2 samples ≥10: {counts:?}");
    }
}
