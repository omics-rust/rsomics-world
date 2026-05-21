use std::path::Path;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-view"))
}

fn fixture() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/small.bam"
    ))
}

#[test]
fn view_all() {
    let out = bin().arg(fixture()).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!out.stdout.is_empty());
}

#[test]
fn count_only() {
    let out = bin().arg("-c").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert_eq!(s.trim(), "10");
}

#[test]
fn filter_by_flags() {
    let out = bin()
        .args(["-f", "1"])
        .arg("-c")
        .arg(fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let n: u64 = s.trim().parse().unwrap();
    assert!(n <= 10);
}
