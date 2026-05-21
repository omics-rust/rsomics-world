use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-fasta-mask"))
}

fn golden(name: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), name)
}

#[test]
fn soft_mask() {
    let out = bin()
        .arg(golden("small.fa"))
        .args(["-b", &golden("mask.bed")])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains('>'));
}

#[test]
fn hard_mask() {
    let out = bin()
        .arg(golden("small.fa"))
        .args(["-b", &golden("mask.bed"), "--hard"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains('N'));
}
