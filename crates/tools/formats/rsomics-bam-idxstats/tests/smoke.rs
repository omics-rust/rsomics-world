use std::path::Path;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-idxstats"))
}

fn fixture() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/small.bam"
    ))
}

#[test]
fn idxstats() {
    let dir = std::env::temp_dir().join("rsomics-bam-idxstats-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let bam = dir.join("test.bam");
    std::fs::copy(fixture(), &bam).unwrap();

    let idx = Command::new("samtools")
        .args(["index", bam.to_str().unwrap()])
        .status();
    if idx.is_err() || !idx.unwrap().success() {
        eprintln!("skipping idxstats: samtools not available");
        let _ = std::fs::remove_dir_all(&dir);
        return;
    }

    let out = bin().arg(&bam).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains('\t'));
    assert!(!s.trim().is_empty());

    let _ = std::fs::remove_dir_all(&dir);
}
