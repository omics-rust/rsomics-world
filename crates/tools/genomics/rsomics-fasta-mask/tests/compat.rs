use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn ours() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_rsomics-fasta-mask"))
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name)
}

fn bedtools_available() -> bool {
    Command::new("bedtools")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn run_cmd(bin: &str, args: &[&str]) -> String {
    let out = Command::new(bin).args(args).output().expect("spawn");
    assert!(
        out.status.success(),
        "{bin} {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf-8")
}

fn run_ours(args: &[&str]) -> String {
    let out = Command::new(ours()).args(args).output().expect("spawn");
    assert!(
        out.status.success(),
        "ours {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf-8")
}

fn parse_fasta_seqs(fasta: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let mut name = String::new();
    let mut seq = String::new();
    for line in fasta.lines() {
        if let Some(n) = line.strip_prefix('>') {
            if !name.is_empty() {
                map.insert(name.clone(), seq.clone());
            }
            name = n.to_string();
            seq.clear();
        } else {
            seq.push_str(line);
        }
    }
    if !name.is_empty() {
        map.insert(name, seq);
    }
    map
}

#[test]
fn soft_mask_matches_bedtools() {
    if !bedtools_available() {
        eprintln!("skipping: bedtools not found");
        return;
    }
    let fa = fixture("small.fa");
    let bed = fixture("mask.bed");

    let ours_raw = run_ours(&[fa.to_str().unwrap(), "-b", bed.to_str().unwrap()]);
    let theirs_raw = run_cmd(
        "bedtools",
        &[
            "maskfasta",
            "-fi",
            fa.to_str().unwrap(),
            "-bed",
            bed.to_str().unwrap(),
            "-fo",
            "/dev/stdout",
            "-soft",
        ],
    );

    let ours = parse_fasta_seqs(&ours_raw);
    let theirs = parse_fasta_seqs(&theirs_raw);

    for (name, our_seq) in &ours {
        let their_seq = theirs
            .get(name)
            .unwrap_or_else(|| panic!("missing seq {name} in bedtools output"));
        assert_eq!(our_seq, their_seq, "soft-mask mismatch on {name}");
    }
}

#[test]
fn hard_mask_matches_bedtools() {
    if !bedtools_available() {
        eprintln!("skipping: bedtools not found");
        return;
    }
    let fa = fixture("small.fa");
    let bed = fixture("mask.bed");

    let ours_raw = run_ours(&[fa.to_str().unwrap(), "-b", bed.to_str().unwrap(), "--hard"]);
    let theirs_raw = run_cmd(
        "bedtools",
        &[
            "maskfasta",
            "-fi",
            fa.to_str().unwrap(),
            "-bed",
            bed.to_str().unwrap(),
            "-fo",
            "/dev/stdout",
        ],
    );

    let ours = parse_fasta_seqs(&ours_raw);
    let theirs = parse_fasta_seqs(&theirs_raw);

    for (name, our_seq) in &ours {
        let their_seq = theirs
            .get(name)
            .unwrap_or_else(|| panic!("missing seq {name} in bedtools output"));
        assert_eq!(our_seq, their_seq, "hard-mask mismatch on {name}");
    }
}
