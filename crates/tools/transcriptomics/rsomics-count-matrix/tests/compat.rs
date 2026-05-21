use std::process::Command;
fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-count-matrix"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn header_has_sample_names() {
    let out = Command::new(ours())
        .arg(golden("s1.txt"))
        .arg(golden("s2.txt"))
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    let header = s.lines().next().unwrap();
    assert!(
        header.contains("s1"),
        "header should contain sample name s1"
    );
    assert!(
        header.contains("s2"),
        "header should contain sample name s2"
    );
}

#[test]
fn count_values_are_preserved() {
    let out = Command::new(ours())
        .arg(golden("s1.txt"))
        .arg(golden("s2.txt"))
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    // BRCA1 had 100 in s1, 80 in s2
    let brca_line = s.lines().find(|l| l.starts_with("BRCA1")).unwrap();
    assert!(brca_line.contains("100"), "BRCA1 should have 100 from s1");
    assert!(brca_line.contains("80"), "BRCA1 should have 80 from s2");
}
