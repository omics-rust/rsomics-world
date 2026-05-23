//! Byte-exact compat against `samtools cat`.
//!
//! cat block-copies the alignment records verbatim, so the records are
//! byte-identical to samtools by construction; the test confirms that and that
//! the merged header bytes match. Both sides run `--no-PG` so the only header
//! difference samtools would otherwise inject (its own @PG with version + CL)
//! is suppressed, leaving the comparison exact.
//!
//! cat/reheader output is stable across samtools releases (it is a raw block
//! copy, not a re-encode), but the test still version-gates: the apt samtools on
//! CI (1.19.2) and the mac dev samtools (1.23.1) both produce identical cat
//! output, and gating keeps the test from false-failing on a samtools too old
//! to have `cat --no-PG` (added in 1.x; >=1.10 is safe).

use std::path::{Path, PathBuf};
use std::process::Command;

fn ours() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-cat"))
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
    eprintln!("SKIP cat compat: samtools {num} (need >= 1.10 for `cat --no-PG`)");
    false
}

fn run_ok(cmd: &mut Command) {
    let status = cmd.status().unwrap();
    assert!(status.success(), "command failed: {cmd:?}");
}

/// SAM record lines (no header) emitted by `samtools view`.
fn records(bam: &Path) -> String {
    let out = Command::new("samtools")
        .arg("view")
        .arg(bam)
        .output()
        .unwrap();
    assert!(out.status.success(), "samtools view failed on {bam:?}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// The BAM's stored header text (`@…` lines) without re-adding a view @PG.
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
fn cat_matches_samtools() {
    if !samtools_ready() {
        return;
    }

    let dir = std::env::temp_dir().join("rsomics-bam-cat-compat");
    let _ = std::fs::create_dir_all(&dir);

    // Two BAMs with identical headers from independent golden SAMs.
    let a = dir.join("a.bam");
    let b = dir.join("b.bam");
    run_ok(
        Command::new("samtools")
            .args(["view", "-b", "--no-PG", "-o"])
            .arg(&a)
            .arg(golden("part_a.sam")),
    );
    run_ok(
        Command::new("samtools")
            .args(["view", "-b", "--no-PG", "-o"])
            .arg(&b)
            .arg(golden("part_b.sam")),
    );

    let sam_out = dir.join("samtools.bam");
    run_ok(
        Command::new("samtools")
            .args(["cat", "--no-PG", "-o"])
            .arg(&sam_out)
            .arg(&a)
            .arg(&b),
    );

    let our_out = dir.join("ours.bam");
    run_ok(ours().args(["--no-PG", "-o"]).arg(&our_out).arg(&a).arg(&b));

    assert_eq!(
        records(&sam_out),
        records(&our_out),
        "cat records differ from samtools"
    );
    assert_eq!(
        header_text(&sam_out),
        header_text(&our_out),
        "cat header differs from samtools"
    );
}
