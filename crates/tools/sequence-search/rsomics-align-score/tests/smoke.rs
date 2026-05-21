use std::process::Command;
fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-align-score"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn global_align() {
    let out = bin().arg(golden("seqs.fa")).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("seq1\tseq2\tscore\tidentity") || s.contains("s1\ts2"));
    let data: Vec<&str> = s.trim().lines().skip(1).collect();
    assert_eq!(data.len(), 3); // 3 pairs from 3 seqs
}
