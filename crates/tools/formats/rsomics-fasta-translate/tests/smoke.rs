use std::path::Path;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-fasta-translate"))
}

fn fixture() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/small.fa"
    ))
}

#[test]
fn frame1() {
    let out = bin().arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let headers: Vec<&str> = s.lines().filter(|l| l.starts_with('>')).collect();
    assert_eq!(headers.len(), 5);
}

#[test]
fn six_frames() {
    let out = bin().args(["-f", "6"]).arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let headers: Vec<&str> = s.lines().filter(|l| l.starts_with('>')).collect();
    assert_eq!(headers.len(), 30);
}

#[test]
fn specific_frames() {
    let out = bin()
        .args(["-f", "1,3,-2"])
        .arg(fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let headers: Vec<&str> = s.lines().filter(|l| l.starts_with('>')).collect();
    assert_eq!(headers.len(), 15);
}
