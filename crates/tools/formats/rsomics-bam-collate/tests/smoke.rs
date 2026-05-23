use std::path::Path;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-collate"))
}

fn fixture() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/scrambled.bam"
    ))
}

#[test]
fn collate_writes_output() {
    let dir = std::env::temp_dir().join("rsomics-bam-collate-smoke");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let output = dir.join("collated.bam");

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
    assert!(err.contains("collated"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn collate_uncompressed_flag_writes_output() {
    let dir = std::env::temp_dir().join("rsomics-bam-collate-smoke-u");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let output = dir.join("collated_u.bam");

    let out = bin()
        .arg(fixture())
        .arg("-u")
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

    let _ = std::fs::remove_dir_all(&dir);
}
