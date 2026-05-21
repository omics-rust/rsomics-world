use std::path::PathBuf;
use std::process::{Command, Stdio};

fn ours() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_rsomics-fasta-utils"))
}

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/small.fa")
}

fn seqkit_available() -> bool {
    Command::new("seqkit")
        .arg("version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn run(bin: &str, args: &[&str]) -> String {
    let out = Command::new(bin).args(args).output().expect("spawn");
    assert!(
        out.status.success(),
        "{bin} {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf-8")
}

fn run_bin(args: &[&str]) -> String {
    let out = Command::new(ours()).args(args).output().expect("spawn");
    assert!(
        out.status.success(),
        "ours {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf-8")
}

#[test]
fn count_matches_seqkit() {
    if !seqkit_available() {
        eprintln!("skipping: seqkit not found");
        return;
    }
    let f = fixture();
    let ours = run_bin(&["count", f.to_str().unwrap()]);
    let theirs = run("seqkit", &["stats", "-T", f.to_str().unwrap()]);
    let seqkit_count: &str = theirs.lines().nth(1).unwrap().split('\t').nth(3).unwrap();
    assert_eq!(ours.trim(), seqkit_count, "record count mismatch");
}

#[test]
fn head_matches_seqkit() {
    if !seqkit_available() {
        eprintln!("skipping: seqkit not found");
        return;
    }
    let f = fixture();
    let ours = run_bin(&["head", "-n", "2", f.to_str().unwrap()]);
    let theirs = run("seqkit", &["head", "-n", "2", f.to_str().unwrap()]);
    let our_names: Vec<&str> = ours.lines().filter(|l| l.starts_with('>')).collect();
    let their_names: Vec<&str> = theirs.lines().filter(|l| l.starts_with('>')).collect();
    assert_eq!(our_names, their_names, "head names mismatch");
}

#[test]
fn revcomp_matches_seqkit() {
    if !seqkit_available() {
        eprintln!("skipping: seqkit not found");
        return;
    }
    let f = fixture();
    let ours = run_bin(&["revcomp", f.to_str().unwrap()]);
    let theirs = run("seqkit", &["seq", "-rp", f.to_str().unwrap()]);
    let our_seqs: Vec<&str> = ours.lines().filter(|l| !l.starts_with('>')).collect();
    let their_seqs: Vec<&str> = theirs.lines().filter(|l| !l.starts_with('>')).collect();
    for (ours, theirs) in our_seqs.iter().zip(their_seqs.iter()) {
        assert_eq!(
            ours.to_uppercase(),
            theirs.to_uppercase(),
            "revcomp mismatch"
        );
    }
}
