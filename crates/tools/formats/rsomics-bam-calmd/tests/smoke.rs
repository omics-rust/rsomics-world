use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-calmd"))
}

fn golden(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name)
}

fn samtools_available() -> bool {
    Command::new("samtools")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

#[test]
fn calmd_smoke() {
    if !samtools_available() {
        eprintln!("skipping: samtools not found");
        return;
    }

    let dir = std::env::temp_dir().join("rsomics-bam-calmd-smoke");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let reference = dir.join("ref.fa");
    std::fs::copy(golden("ref.fa"), &reference).unwrap();
    std::fs::copy(golden("ref.fa.fai"), dir.join("ref.fa.fai")).unwrap();

    let bam = dir.join("in.bam");
    assert!(
        Command::new("samtools")
            .args(["sort", "-o"])
            .arg(&bam)
            .arg(golden("calmd_in.sam"))
            .status()
            .unwrap()
            .success(),
        "samtools sort failed"
    );

    let output = dir.join("calmd.bam");
    let out = bin()
        .arg(&bam)
        .arg(&reference)
        .arg("-o")
        .arg(&output)
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(output.exists(), "output BAM not created");

    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("recomputed"),
        "expected stats on stderr, got: {err}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
