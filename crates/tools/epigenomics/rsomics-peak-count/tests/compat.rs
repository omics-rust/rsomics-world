use std::process::Command;
fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-peak-count"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn counts_are_non_negative() {
    let out = Command::new(ours())
        .arg(golden("small.bam"))
        .args(["-b", &golden("peaks.bed")])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    for line in s.lines() {
        let count: u64 = line.split('\t').nth(3).unwrap().parse().unwrap();
        assert!(count < 1_000_000, "count should be reasonable: {count}");
    }
}
