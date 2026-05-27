use std::path::PathBuf;
use std::process::{Command, Stdio};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name)
}

fn ours() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_rsomics-deletion-profile"))
}

fn deletion_profile_available() -> bool {
    Command::new("deletion_profile.py")
        .arg("--help")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// Run deletion_profile.py, assert success, return its .deletion_profile.txt contents.
fn run_upstream(bam: &str, out_prefix: &str, read_length: &str, mapq: &str) -> String {
    let status = Command::new("deletion_profile.py")
        .args(["-i", bam, "-l", read_length, "-o", out_prefix, "-q", mapq])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("spawn deletion_profile.py");
    assert!(
        status.success(),
        "deletion_profile.py failed with code {:?}",
        status.code()
    );
    let txt_path = format!("{out_prefix}.deletion_profile.txt");
    std::fs::read_to_string(&txt_path).unwrap_or_else(|e| panic!("reading {txt_path}: {e}"))
}

#[test]
fn deletion_profile_matches_upstream() {
    if !deletion_profile_available() {
        eprintln!("SKIP: deletion_profile.py not on PATH");
        return;
    }

    let bam = fixture("del_test.bam");
    let bam_str = bam.to_str().unwrap();

    let tmp = tempfile::tempdir().unwrap();
    let ours_prefix = tmp.path().join("ours").to_string_lossy().into_owned();
    let theirs_prefix = tmp.path().join("theirs").to_string_lossy().into_owned();

    // Run ours — use --mapq 0 so all reads in the golden fixture (mapq 60) pass;
    // upstream uses -q 0 for the same effect (apples-to-apples comparison).
    let our_status = Command::new(ours())
        .args([
            "-i",
            bam_str,
            "-l",
            "100",
            "-o",
            &ours_prefix,
            "--mapq",
            "0",
        ])
        .status()
        .expect("spawn ours");
    assert!(
        our_status.success(),
        "rsomics-deletion-profile failed with code {:?}",
        our_status.code()
    );

    let ours_txt = std::fs::read_to_string(format!("{ours_prefix}.deletion_profile.txt"))
        .expect("reading ours txt");

    let theirs_txt = run_upstream(bam_str, &theirs_prefix, "100", "0");

    assert_eq!(
        ours_txt, theirs_txt,
        "deletion_profile.txt mismatch:\nours:\n{ours_txt}\ntheirs:\n{theirs_txt}"
    );
}

#[test]
fn ours_exits_zero_on_valid_input() {
    let bam = fixture("del_test.bam");
    let tmp = tempfile::tempdir().unwrap();
    let prefix = tmp.path().join("out").to_string_lossy().into_owned();

    let status = Command::new(ours())
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-l",
            "100",
            "-o",
            &prefix,
            "--mapq",
            "0",
        ])
        .status()
        .expect("spawn ours");
    assert!(status.success());

    let txt = std::fs::read_to_string(format!("{prefix}.deletion_profile.txt")).unwrap();
    assert!(txt.starts_with("read_position\tdeletion_count\n"));

    // positions 30 and 50 should have count 1
    let non_zero: Vec<&str> = txt
        .lines()
        .skip(1)
        .filter(|l| !l.ends_with('\t') && l.split('\t').nth(1).is_some_and(|v| v != "0"))
        .collect();
    assert_eq!(non_zero, ["30\t1", "50\t1"]);
}

#[test]
fn ours_exits_nonzero_on_missing_input() {
    let tmp = tempfile::tempdir().unwrap();
    let prefix = tmp.path().join("out").to_string_lossy().into_owned();

    let status = Command::new(ours())
        .args(["-i", "/nonexistent.bam", "-l", "100", "-o", &prefix])
        .status()
        .expect("spawn ours");
    assert!(!status.success());
}
