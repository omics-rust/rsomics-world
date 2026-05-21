use std::path::Path;
use std::process::Command;
fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-fastq-downsample"))
}
fn fixture() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/small.fq"
    ))
}

#[test]
fn downsample_all() {
    let out = bin()
        .arg(fixture())
        .args(["-f", "1.0", "--seed", "42"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    let names: Vec<&str> = s.lines().filter(|l| l.starts_with('@')).collect();
    assert_eq!(names.len(), 4);
}
