use std::path::Path;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-fastq-validate"))
}

fn fixture() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/small.fq"
    ))
}

#[test]
fn valid_fastq() {
    let out = bin().arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("OK"));
    assert!(err.contains("4 records"));
}

#[test]
fn invalid_fastq() {
    let dir = std::env::temp_dir().join("rsomics-fastq-validate-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let bad = dir.join("bad.fq");
    std::fs::write(&bad, "@read1\nACGT\n+\nIII\n").unwrap();
    let out = bin().arg(&bad).output().unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("INVALID"));
    let _ = std::fs::remove_dir_all(&dir);
}
