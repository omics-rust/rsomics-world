use std::path::Path;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-fasta-validate"))
}

fn fixture() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/small.fa"
    ))
}

#[test]
fn valid_fasta() {
    let out = bin().arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("OK"));
    assert!(err.contains("5 sequences"));
}

#[test]
fn invalid_fasta() {
    let dir = std::env::temp_dir().join("rsomics-fasta-validate-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let bad = dir.join("bad.fa");
    std::fs::write(&bad, "not a fasta file\nAAAA\n").unwrap();
    let out = bin().arg(&bad).output().unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("INVALID"));
    let _ = std::fs::remove_dir_all(&dir);
}
