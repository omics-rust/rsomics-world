use std::process::Command;
fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-kraken-report"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn species_are_sorted_by_reads() {
    let out = Command::new(ours())
        .arg(golden("report.txt"))
        .args(["-r", "S", "-n", "10"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    let reads: Vec<u64> = s
        .lines()
        .skip(1)
        .filter_map(|l| l.split('\t').nth(1)?.parse().ok())
        .collect();
    for w in reads.windows(2) {
        assert!(w[0] >= w[1], "not sorted: {} < {}", w[0], w[1]);
    }
}
