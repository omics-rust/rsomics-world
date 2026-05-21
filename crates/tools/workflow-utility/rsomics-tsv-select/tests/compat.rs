use std::process::Command;

fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-tsv-select"))
}

fn fixture() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/data.tsv")
}

#[test]
fn select_matches_cut() {
    // Our "select by index 1,3" should match `cut -f1,3`
    let our_out = Command::new(ours())
        .arg(fixture())
        .args(["-c", "1", "3"])
        .output()
        .unwrap();
    assert!(our_out.status.success());
    let ours = String::from_utf8(our_out.stdout).unwrap();

    let cut_out = Command::new("cut")
        .args(["-f1,3", fixture().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(cut_out.status.success());
    let theirs = String::from_utf8(cut_out.stdout).unwrap();

    assert_eq!(ours, theirs, "column selection should match cut -f1,3");
}

#[test]
fn preserves_row_count() {
    let our_out = Command::new(ours())
        .arg(fixture())
        .args(["-c", "gene"])
        .output()
        .unwrap();
    assert!(our_out.status.success());
    let ours = String::from_utf8(our_out.stdout).unwrap();

    let wc_out = Command::new("wc")
        .args(["-l", fixture().to_str().unwrap()])
        .output()
        .unwrap();
    let wc_str = String::from_utf8(wc_out.stdout).unwrap();
    let original_lines: usize = wc_str
        .split_whitespace()
        .next()
        .unwrap()
        .parse()
        .unwrap();

    let our_lines = ours.lines().count();
    assert_eq!(our_lines, original_lines, "row count must be preserved");
}
