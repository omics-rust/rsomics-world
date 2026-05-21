use std::path::PathBuf;
use std::process::{Command, Stdio};

fn ours() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_rsomics-bam-subsample"))
}

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/small.bam")
}

fn samtools_available() -> bool {
    Command::new("samtools")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn count_bam(path: &std::path::Path) -> u64 {
    let out = Command::new("samtools")
        .args(["view", "-c", path.to_str().unwrap()])
        .output()
        .expect("samtools");
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse()
        .unwrap_or(0)
}

#[test]
fn fraction_1_keeps_all() {
    if !samtools_available() {
        eprintln!("skipping: samtools not found");
        return;
    }

    let dir = std::env::temp_dir().join("rsomics-bam-subsample-compat");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let output = dir.join("sub.bam");

    let out = Command::new(ours())
        .arg(fixture())
        .args(["-f", "1.0", "--seed", "42", "-o"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(out.status.success());

    let original = count_bam(&fixture());
    let subsampled = count_bam(&output);
    assert_eq!(original, subsampled, "fraction=1.0 should keep all reads");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fraction_half_reduces_count() {
    if !samtools_available() {
        eprintln!("skipping: samtools not found");
        return;
    }

    let dir = std::env::temp_dir().join("rsomics-bam-subsample-compat-half");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let output = dir.join("sub.bam");

    let out = Command::new(ours())
        .arg(fixture())
        .args(["-f", "0.5", "--seed", "42", "-o"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(out.status.success());

    let original = count_bam(&fixture());
    let subsampled = count_bam(&output);
    assert!(
        subsampled < original,
        "fraction=0.5 should reduce count: {subsampled} vs {original}"
    );
    assert!(subsampled > 0, "should keep at least some reads");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn deterministic_with_same_seed() {
    if !samtools_available() {
        eprintln!("skipping: samtools not found");
        return;
    }

    let dir = std::env::temp_dir().join("rsomics-bam-subsample-deterministic");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let out1 = dir.join("run1.bam");
    let out2 = dir.join("run2.bam");

    for output in [&out1, &out2] {
        let out = Command::new(ours())
            .arg(fixture())
            .args(["-f", "0.5", "--seed", "123", "-o"])
            .arg(output)
            .output()
            .unwrap();
        assert!(out.status.success());
    }

    let c1 = count_bam(&out1);
    let c2 = count_bam(&out2);
    assert_eq!(c1, c2, "same seed should produce same count");

    let _ = std::fs::remove_dir_all(&dir);
}
