use std::process::Command;
fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-fasta-digest"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn trypsin_digest() {
    let out = bin()
        .arg(golden("protein.fa"))
        .args(["-e", "trypsin", "--min-len", "3", "--max-len", "50"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains('\t'));
    assert!(!s.is_empty());
}
