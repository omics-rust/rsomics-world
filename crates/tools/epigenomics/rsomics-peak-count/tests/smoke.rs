use std::process::Command;
fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-peak-count"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn count_reads_in_peaks() {
    let out = bin()
        .arg(golden("small.bam"))
        .args(["-b", &golden("peaks.bed")])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains('\t'));
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 2);
}
