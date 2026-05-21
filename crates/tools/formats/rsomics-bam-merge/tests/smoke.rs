use std::path::Path;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-merge"))
}

fn fixture() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/small.bam"
    ))
}

#[test]
fn merge_same_file() {
    let dir = std::env::temp_dir().join("rsomics-bam-merge-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let output = dir.join("merged.bam");

    let out = bin()
        .arg(fixture())
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
    assert!(output.metadata().unwrap().len() > 0);

    let _ = std::fs::remove_dir_all(&dir);
}
