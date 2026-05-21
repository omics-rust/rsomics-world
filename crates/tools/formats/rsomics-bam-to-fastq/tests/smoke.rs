use std::path::Path;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-to-fastq"))
}

fn fixture() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/small.bam"
    ))
}

#[test]
fn to_fastq() {
    let out = bin().arg(fixture()).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    let headers: Vec<&str> = s.lines().filter(|l| l.starts_with('@')).collect();
    assert!(!headers.is_empty());
}
