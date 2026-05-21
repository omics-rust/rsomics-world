use std::process::Command;
fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-de-volcano"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn annotate() {
    let out = bin().arg(golden("de.tsv")).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("UP"));
    assert!(s.contains("DOWN"));
    assert!(s.contains("NS"));
}
