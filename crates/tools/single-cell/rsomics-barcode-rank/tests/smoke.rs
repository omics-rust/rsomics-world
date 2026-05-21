use std::process::Command;
fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-barcode-rank"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn rank_output() {
    let out = bin().arg(golden("counts.tsv")).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("rank\tcount"));
    let data: Vec<&str> = s.trim().lines().skip(1).collect();
    assert_eq!(data.len(), 5);
    // First rank should be highest count
    assert!(data[0].contains("5000"));
}
