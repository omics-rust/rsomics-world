use std::process::Command;
fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-deseq-prep"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn filter_low_counts() {
    let out = bin()
        .arg(golden("counts.tsv"))
        .args(["--min-count", "10", "--min-samples", "2"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    let data: Vec<&str> = s.trim().lines().skip(1).collect();
    assert_eq!(data.len(), 2); // A and C pass, B filtered
    assert!(s.contains("gene\t"));
}
