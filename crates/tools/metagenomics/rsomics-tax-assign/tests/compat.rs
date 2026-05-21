use std::process::Command;
fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-tax-assign"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn output_has_correct_format() {
    let out = Command::new(ours())
        .arg(golden("reads.fq"))
        .args(["-d", &golden("empty_db.tsv"), "-k", "5"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    for line in s.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            parts.len(),
            4,
            "each line should have 4 fields: C/U, name, taxid, count"
        );
        assert!(
            parts[0] == "C" || parts[0] == "U",
            "first field must be C or U"
        );
    }
}
