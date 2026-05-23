//! Byte-exact compat against `samtools reheader`.
//!
//! reheader block-copies the alignment records verbatim, so they are
//! byte-identical to samtools by construction; the test confirms that and that
//! the replacement-header bytes match. Both sides run `--no-PG` so samtools'
//! own @PG injection is suppressed, leaving the header comparison exact.
//!
//! Version-gated >= 1.10 (stable raw-block reheader path).

use std::path::{Path, PathBuf};
use std::process::Command;

fn ours() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-reheader"))
}

fn golden(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name)
}

fn samtools_ready() -> bool {
    let Ok(out) = Command::new("samtools").arg("--version").output() else {
        return false;
    };
    if !out.status.success() {
        return false;
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let num = stdout
        .lines()
        .next()
        .unwrap_or("")
        .split_whitespace()
        .nth(1)
        .unwrap_or("");
    let mut it = num.split('.');
    let major: u32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor: u32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    if major > 1 || (major == 1 && minor >= 10) {
        return true;
    }
    eprintln!("SKIP reheader compat: samtools {num} (need >= 1.10)");
    false
}

fn run_ok(cmd: &mut Command) {
    let status = cmd.status().unwrap();
    assert!(status.success(), "command failed: {cmd:?}");
}

fn records(bam: &Path) -> String {
    let out = Command::new("samtools")
        .arg("view")
        .arg(bam)
        .output()
        .unwrap();
    assert!(out.status.success(), "samtools view failed on {bam:?}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn header_text(bam: &Path) -> String {
    let out = Command::new("samtools")
        .args(["head"])
        .arg(bam)
        .output()
        .unwrap();
    assert!(out.status.success(), "samtools head failed on {bam:?}");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.starts_with("@PG"))
        .map(|l| format!("{l}\n"))
        .collect()
}

#[test]
fn reheader_matches_samtools() {
    if !samtools_ready() {
        return;
    }

    let dir = std::env::temp_dir().join("rsomics-bam-reheader-compat");
    let _ = std::fs::create_dir_all(&dir);

    let bam = dir.join("in.bam");
    run_ok(
        Command::new("samtools")
            .args(["view", "-b", "--no-PG", "-o"])
            .arg(&bam)
            .arg(golden("reads.sam")),
    );

    let new_header = golden("new_header.sam");

    // samtools reheader writes to stdout.
    let sam_out = dir.join("samtools.bam");
    let out = Command::new("samtools")
        .args(["reheader", "--no-PG"])
        .arg(&new_header)
        .arg(&bam)
        .output()
        .unwrap();
    assert!(out.status.success(), "samtools reheader failed");
    std::fs::write(&sam_out, &out.stdout).unwrap();

    let our_out = dir.join("ours.bam");
    run_ok(
        ours()
            .args(["--no-PG", "-o"])
            .arg(&our_out)
            .arg(&new_header)
            .arg(&bam),
    );

    assert_eq!(
        records(&sam_out),
        records(&our_out),
        "reheader records differ from samtools"
    );
    assert_eq!(
        header_text(&sam_out),
        header_text(&our_out),
        "reheader header differs from samtools"
    );
}
