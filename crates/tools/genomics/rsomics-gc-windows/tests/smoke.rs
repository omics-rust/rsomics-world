use std::path::Path;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-gc-windows"))
}

fn fixture() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/small.fa"
    ))
}

#[test]
fn basic_output() {
    let out = bin().args(["-w", "5"]).arg(fixture()).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains('\t'));
    let lines: Vec<&str> = s.trim().lines().collect();
    assert!(!lines.is_empty());
}
