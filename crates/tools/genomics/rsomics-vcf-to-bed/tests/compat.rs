use std::process::Command;
fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-vcf-to-bed"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn bed_is_0_based() {
    // VCF is 1-based, BED is 0-based: VCF pos 100 → BED start 99
    let out = Command::new(ours())
        .arg(golden("small.vcf"))
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    let first_line = s.lines().next().unwrap();
    let start: u64 = first_line.split('\t').nth(1).unwrap().parse().unwrap();
    assert_eq!(start, 99, "VCF pos=100 should become BED start=99");
}

#[test]
fn end_accounts_for_ref_length() {
    let out = Command::new(ours())
        .arg(golden("small.vcf"))
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    for line in s.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        let start: u64 = parts[1].parse().unwrap();
        let end: u64 = parts[2].parse().unwrap();
        assert!(end > start, "end must be > start: {start}-{end}");
    }
}
