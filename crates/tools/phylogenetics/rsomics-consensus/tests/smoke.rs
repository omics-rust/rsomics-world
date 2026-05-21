use std::process::Command;
fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-consensus"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn majority_consensus() {
    let out = bin()
        .arg(golden("aln.fa"))
        .args(["--threshold", "0.5"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains(">consensus"));
    // Position 8: 2×G, 1×C → majority is G at 0.67 > 0.5
    let seq: String = s.lines().filter(|l| !l.starts_with('>')).collect();
    assert_eq!(seq.len(), 8);
    assert!(seq.starts_with("ATCGATC"));
}
