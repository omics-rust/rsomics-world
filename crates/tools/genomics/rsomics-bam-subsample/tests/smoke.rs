use std::path::Path;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-subsample"))
}

fn fixture() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/small.bam"
    ))
}

#[test]
fn subsample_fraction() {
    let dir = std::env::temp_dir().join("rsomics-bam-subsample-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let output = dir.join("sub.bam");

    let out = bin()
        .arg(fixture())
        .args(["-f", "1.0", "-o"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(output.exists());
    assert!(output.metadata().unwrap().len() > 0);
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("10 records kept"));

    let _ = std::fs::remove_dir_all(&dir);
}
