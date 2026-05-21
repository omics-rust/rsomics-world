use std::process::Command;
fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-tsv-join"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn inner_join() {
    let out = bin()
        .arg(golden("left.tsv"))
        .arg(golden("right.tsv"))
        .args(["-k", "gene"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    let data: Vec<&str> = s.trim().lines().skip(1).collect();
    assert_eq!(data.len(), 2); // BRCA1 + EGFR (TP53 not in right, MYC not in left)
    assert!(s.contains("BRCA1"));
    assert!(s.contains("chr17"));
}
