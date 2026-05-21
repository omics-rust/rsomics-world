use std::path::Path;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-index"))
}

fn fixture() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/small.bam"
    ))
}

#[test]
fn create_index() {
    let dir = std::env::temp_dir().join("rsomics-bam-index-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let bam = dir.join("test.bam");
    std::fs::copy(fixture(), &bam).unwrap();

    let out = bin().arg(&bam).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let bai = dir.join("test.bam.bai");
    assert!(bai.exists(), "index file should be created");
    assert!(bai.metadata().unwrap().len() > 0);

    let _ = std::fs::remove_dir_all(&dir);
}
