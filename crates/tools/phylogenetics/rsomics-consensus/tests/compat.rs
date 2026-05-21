use std::process::Command;

fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-consensus"))
}

fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

fn parse_seq(fasta: &str) -> String {
    fasta.lines().filter(|l| !l.starts_with('>')).collect()
}

#[test]
fn unanimous_positions_are_exact() {
    // All 3 seqs agree on positions 1-7 (ATCGATC), differ at position 8
    let out = Command::new(ours())
        .arg(golden("aln.fa"))
        .args(["--threshold", "0.5"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let seq = parse_seq(&String::from_utf8(out.stdout).unwrap());
    // First 7 positions are unanimous
    assert!(seq.starts_with("ATCGATC"), "consensus: {seq}");
}

#[test]
fn threshold_1_requires_unanimity() {
    let out = Command::new(ours())
        .arg(golden("aln.fa"))
        .args(["--threshold", "1.0"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let seq = parse_seq(&String::from_utf8(out.stdout).unwrap());
    // Position 8: 2×G, 1×C → not unanimous → N
    assert_eq!(
        &seq[7..8],
        "N",
        "non-unanimous position should be N at threshold=1.0"
    );
}

#[test]
fn output_length_equals_alignment_length() {
    let out = Command::new(ours())
        .arg(golden("aln.fa"))
        .args(["--threshold", "0.5"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let seq = parse_seq(&String::from_utf8(out.stdout).unwrap());
    assert_eq!(seq.len(), 8, "consensus length must match alignment length");
}
