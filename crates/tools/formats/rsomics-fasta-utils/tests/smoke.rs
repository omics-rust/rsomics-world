use std::path::Path;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-fasta-utils"))
}

fn fixture() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/small.fa"
    ))
}

#[test]
fn count() {
    let out = bin().arg("count").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "5");
}

#[test]
fn chroms() {
    let out = bin().arg("chroms").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 5);
    assert!(lines.contains(&"seq1"));
}

#[test]
fn composition() {
    let out = bin().arg("composition").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("A\t"));
    assert!(s.contains("total\t"));
}

#[test]
fn gc_content() {
    let out = bin().arg("gc-content").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 6);
    assert!(lines[0].starts_with("name\t"));
}

#[test]
fn len() {
    let out = bin().arg("len").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 5);
    assert_eq!(lines[0], "12");
}

#[test]
fn len_tab() {
    let out = bin()
        .args(["len", "--tab"])
        .arg(fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("seq1\t12"));
}

#[test]
fn revcomp() {
    let out = bin().arg("revcomp").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains(">seq1"));
    assert!(s.contains("CGATCGATCGAT"));
}

#[test]
fn upper() {
    let out = bin().arg("upper").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("ATCG"));
    assert!(!s.lines().any(|l| !l.starts_with('>') && l.chars().any(|c| c.is_ascii_lowercase())));
}

#[test]
fn tab() {
    let out = bin().arg("tab").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 5);
    assert!(lines[0].contains('\t'));
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
    let names: Vec<&str> = s.lines().filter(|l| l.starts_with('>')).collect();
    assert_eq!(names.len(), 2);
}

#[test]
fn grep() {
    let out = bin()
        .args(["grep", "-p", "seq[12]"])
        .arg(fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let names: Vec<&str> = s.lines().filter(|l| l.starts_with('>')).collect();
    assert_eq!(names.len(), 2);
}

#[test]
fn filter_min_len() {
    let out = bin()
        .args(["filter", "-m", "15"])
        .arg(fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let names: Vec<&str> = s.lines().filter(|l| l.starts_with('>')).collect();
    assert_eq!(names.len(), 3);
}

#[test]
fn sort_by_name() {
    let out = bin().arg("sort").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let names: Vec<&str> = s
        .lines()
        .filter(|l| l.starts_with('>'))
        .map(|l| &l[1..])
        .collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    assert_eq!(names, sorted);
}

#[test]
fn sort_by_length() {
    let out = bin()
        .args(["sort", "-l"])
        .arg(fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
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
fn shuffle() {
    let out = bin()
        .args(["shuffle", "--seed", "42"])
        .arg(fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let names: Vec<&str> = s.lines().filter(|l| l.starts_with('>')).collect();
    assert_eq!(names.len(), 5);
}

#[test]
fn dedup() {
    let out = bin().arg("dedup").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let names: Vec<&str> = s.lines().filter(|l| l.starts_with('>')).collect();
    assert_eq!(names.len(), 5);
}

#[test]
fn kmers() {
    let out = bin()
        .args(["kmers", "-k", "3"])
        .arg(fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("ATC\t"));
}

#[test]
fn to_bed() {
    let out = bin().arg("to-bed").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 5);
    assert!(lines[0].contains("seq1\t0\t12"));
}

#[test]
fn wrap() {
    let out = bin()
        .args(["wrap", "-w", "5"])
        .arg(fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
}
