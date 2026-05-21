use std::process::Command;
fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-count-matrix"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn merge_two_samples() {
    let out = bin()
        .arg(golden("s1.txt"))
        .arg(golden("s2.txt"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("gene\t"));
    assert!(s.contains("BRCA1"));
    assert!(s.contains("TP53"));
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 4); // header + 3 genes
}
