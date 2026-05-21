use std::process::Command;

fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-msa-trim"))
}

fn fixture() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/aln.fa")
}

fn parse_seqs(fasta: &str) -> Vec<(String, String)> {
    let mut result = Vec::new();
    let mut name = String::new();
    let mut seq = String::new();
    for line in fasta.lines() {
        if let Some(n) = line.strip_prefix('>') {
            if !name.is_empty() {
                result.push((name.clone(), seq.clone()));
            }
            name = n.to_string();
            seq.clear();
        } else {
            seq.push_str(line);
        }
    }
    if !name.is_empty() {
        result.push((name, seq));
    }
    result
}

#[test]
fn trimmed_columns_have_low_gap_fraction() {
    let out = Command::new(ours())
        .arg(fixture())
        .args(["-g", "0.3"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    let seqs = parse_seqs(&s);
    assert!(!seqs.is_empty());

    let aln_len = seqs[0].1.len();
    let n = seqs.len();
    for col in 0..aln_len {
        let gaps = seqs
            .iter()
            .filter(|(_, s)| s.as_bytes().get(col).copied() == Some(b'-'))
            .count();
        let frac = gaps as f64 / n as f64;
        assert!(
            frac <= 0.3 + 0.01,
            "column {col} has gap fraction {frac:.2}, expected <= 0.3"
        );
    }
}

#[test]
fn all_seqs_same_length() {
    let out = Command::new(ours())
        .arg(fixture())
        .args(["-g", "0.5"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    let seqs = parse_seqs(&s);
    let lengths: Vec<usize> = seqs.iter().map(|(_, s)| s.len()).collect();
    assert!(
        lengths.windows(2).all(|w| w[0] == w[1]),
        "all sequences must have equal length after trimming: {lengths:?}"
    );
}

#[test]
fn threshold_0_keeps_only_gapless() {
    let out = Command::new(ours())
        .arg(fixture())
        .args(["-g", "0.0"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    let seqs = parse_seqs(&s);
    for (name, seq) in &seqs {
        assert!(
            !seq.contains('-'),
            "seq {name} still has gaps with threshold 0.0"
        );
    }
}
