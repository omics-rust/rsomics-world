use std::path::Path;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-fastq-utils"))
}

fn fixture() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/small.fq"
    ))
}

#[test]
fn count() {
    let out = bin().arg("count").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "4");
}

#[test]
fn head() {
    let out = bin()
        .args(["head", "-n", "2"])
        .arg(fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let names: Vec<&str> = s.lines().filter(|l| l.starts_with('@')).collect();
    assert_eq!(names.len(), 2);
}

#[test]
fn len() {
    let out = bin().arg("len").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains('8'));
    assert!(s.contains("16"));
    assert!(s.contains("12"));
}

#[test]
fn gc() {
    let out = bin().arg("gc").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 5);
    assert!(lines[0].starts_with("read\t"));
}

#[test]
fn revcomp() {
    let out = bin().arg("revcomp").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("CGATCGAT"));
}

#[test]
fn to_fasta() {
    let out = bin().arg("to-fasta").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let names: Vec<&str> = s.lines().filter(|l| l.starts_with('>')).collect();
    assert_eq!(names.len(), 4);
}

#[test]
fn tab() {
    let out = bin().arg("tab").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 4);
    assert!(lines[0].contains('\t'));
}

#[test]
fn grep() {
    let out = bin()
        .args(["grep", "-p", "read[12]"])
        .arg(fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let names: Vec<&str> = s.lines().filter(|l| l.starts_with('@')).collect();
    assert_eq!(names.len(), 2);
}

#[test]
fn sort_by_name() {
    let out = bin().arg("sort").arg(fixture()).output().unwrap();
    assert!(out.status.success());
}

#[test]
fn shuffle() {
    let out = bin()
        .args(["shuffle", "--seed", "42"])
        .arg(fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let names: Vec<&str> = s.lines().filter(|l| l.starts_with('@')).collect();
    assert_eq!(names.len(), 4);
}

#[test]
fn sample() {
    let out = bin()
        .args(["sample", "-p", "1.0", "--seed", "1"])
        .arg(fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
}

#[test]
fn rename() {
    let out = bin()
        .args(["rename", "--prefix", "r_"])
        .arg(fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("@r_0"));
    assert!(s.contains("@r_3"));
}
