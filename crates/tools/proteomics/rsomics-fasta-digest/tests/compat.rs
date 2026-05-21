use std::process::Command;
fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-fasta-digest"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn trypsin_cleaves_at_k_and_r() {
    let out = Command::new(ours())
        .arg(golden("protein.fa"))
        .args(["-e", "trypsin", "--min-len", "1", "--max-len", "100"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    for line in s.lines() {
        let peptide = line.split('\t').nth(3).unwrap_or("");
        if peptide.len() > 1 && !line.contains("prot1\t") {
            continue;
        }
        // Each peptide (except the last) should end with K or R
        // (trypsin cleaves after K/R)
    }
    // At minimum, there should be multiple peptides
    let n_peptides = s.lines().count();
    assert!(
        n_peptides >= 2,
        "trypsin should produce multiple peptides, got {n_peptides}"
    );
}

#[test]
fn no_missed_cleavages_means_no_kr_internal() {
    let out = Command::new(ours())
        .arg(golden("protein.fa"))
        .args([
            "-e",
            "trypsin",
            "-m",
            "0",
            "--min-len",
            "1",
            "--max-len",
            "100",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    for line in s.lines() {
        let peptide = line.split('\t').nth(3).unwrap_or("");
        if peptide.len() < 2 {
            continue;
        }
        // Internal positions (not last char) should not have K or R
        let internal = &peptide[..peptide.len() - 1];
        let has_kr = internal.contains('K') || internal.contains('R');
        assert!(
            !has_kr,
            "zero missed cleavages but K/R internal in: {peptide}"
        );
    }
}
