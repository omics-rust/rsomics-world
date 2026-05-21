use std::process::Command;
fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-msa-trim"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn trim_gaps() {
    let out = bin()
        .arg(golden("aln.fa"))
        .args(["-g", "0.5"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    let seqs: Vec<&str> = s.lines().filter(|l| !l.starts_with('>')).collect();
    assert_eq!(seqs.len(), 3);
    // Columns 3,4 have 2/3 gaps (0.67 > 0.5) → removed
    // Columns 5,6 have 2/3 gaps → removed
    // Remaining: columns 1,2,7,8 = 4 chars each
    assert!(seqs[0].len() < 8);
}
