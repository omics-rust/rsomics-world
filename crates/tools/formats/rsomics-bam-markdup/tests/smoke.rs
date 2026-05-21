use std::path::Path;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-markdup"))
}

fn fixture() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/small.bam"
    ))
}

#[test]
fn markdup() {
    let dir = std::env::temp_dir().join("rsomics-bam-markdup-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let output = dir.join("deduped.bam");

    let out = bin()
        .arg(fixture())
        .arg("-o")
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(output.exists());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("total") || err.contains("marked") || err.contains("processed"));

    let _ = std::fs::remove_dir_all(&dir);
}
