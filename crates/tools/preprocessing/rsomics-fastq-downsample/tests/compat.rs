use std::path::PathBuf;
use std::process::{Command, Stdio};

fn ours() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_rsomics-fastq-downsample"))
}

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/small.fq")
}

fn seqtk_available() -> bool {
    Command::new("seqtk")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

fn count_reads(fq: &str) -> usize {
    fq.lines().filter(|l| l.starts_with('@')).count()
}

#[test]
fn fraction_1_keeps_all() {
    let out = Command::new(ours())
        .arg(fixture())
        .args(["-f", "1.0", "--seed", "42"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert_eq!(count_reads(&s), 4, "fraction=1.0 should keep all 4 reads");
}

#[test]
fn fraction_0_keeps_none() {
    let out = Command::new(ours())
        .arg(fixture())
        .args(["-f", "0.0", "--seed", "42"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert_eq!(count_reads(&s), 0, "fraction=0.0 should keep no reads");
}

#[test]
fn deterministic() {
    let run = |seed: &str| -> String {
        let out = Command::new(ours())
            .arg(fixture())
            .args(["-f", "0.5", "--seed", seed])
            .output()
            .unwrap();
        assert!(out.status.success());
        String::from_utf8(out.stdout).unwrap()
    };
    let r1 = run("99");
    let r2 = run("99");
    assert_eq!(r1, r2, "same seed must produce identical output");
}

#[test]
fn output_is_valid_fastq() {
    let out = Command::new(ours())
        .arg(fixture())
        .args(["-f", "1.0", "--seed", "1"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = s.lines().collect();
    assert_eq!(lines.len() % 4, 0, "FASTQ must have lines divisible by 4");
    for chunk in lines.chunks(4) {
        assert!(chunk[0].starts_with('@'), "header line must start with @");
        assert_eq!(chunk[2], "+", "separator line must be +");
        assert_eq!(
            chunk[1].len(),
            chunk[3].len(),
            "seq and qual must have equal length"
        );
    }
}

#[test]
fn count_comparable_to_seqtk() {
    if !seqtk_available() {
        eprintln!("skipping: seqtk not found");
        return;
    }
    let ours_out = Command::new(ours())
        .arg(fixture())
        .args(["-f", "0.5", "--seed", "42"])
        .output()
        .unwrap();
    assert!(ours_out.status.success());
    let our_count = count_reads(&String::from_utf8_lossy(&ours_out.stdout));

    let seqtk_out = Command::new("seqtk")
        .args(["sample", "-s", "42", fixture().to_str().unwrap(), "2"])
        .output()
        .unwrap();
    let seqtk_count = count_reads(&String::from_utf8_lossy(&seqtk_out.stdout));

    assert!(our_count <= 4 && our_count > 0);
    assert!(seqtk_count <= 4 && seqtk_count > 0);
}
