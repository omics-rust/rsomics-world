//! Byte-exact compat against `samtools head`.
//!
//! head emits SAM text: the stored header (verbatim) plus the first N records
//! formatted by our own `sam_format`. The test diffs full stdout against
//! `samtools head` for the default (header only), `-n` (records), `-h` header
//! truncation, and the combination. Our `-H` is samtools' `-h` (the short `-h`
//! is reserved for help in this workspace), so the test maps it.
//!
//! Version-gated >= 1.13 (when `samtools head` was added).

use std::path::{Path, PathBuf};
use std::process::Command;

fn ours() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-head"))
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
    if major > 1 || (major == 1 && minor >= 13) {
        return true;
    }
    eprintln!("SKIP head compat: samtools {num} (need >= 1.13 for `samtools head`)");
    false
}

fn make_bam(dir: &Path) -> PathBuf {
    let bam = dir.join("in.bam");
    let status = Command::new("samtools")
        .args(["view", "-b", "--no-PG", "-o"])
        .arg(&bam)
        .arg(golden("reads.sam"))
        .status()
        .unwrap();
    assert!(status.success(), "samtools view failed building fixture");
    bam
}

fn samtools_head(bam: &Path, args: &[&str]) -> Vec<u8> {
    let out = Command::new("samtools")
        .arg("head")
        .args(args)
        .arg(bam)
        .output()
        .unwrap();
    assert!(out.status.success(), "samtools head {args:?} failed");
    out.stdout
}

fn our_head(bam: &Path, args: &[&str]) -> Vec<u8> {
    let out = ours().args(args).arg(bam).output().unwrap();
    assert!(out.status.success(), "rsomics-bam-head {args:?} failed");
    out.stdout
}

#[test]
fn head_matches_samtools() {
    if !samtools_ready() {
        return;
    }
    let dir = std::env::temp_dir().join("rsomics-bam-head-compat");
    let _ = std::fs::create_dir_all(&dir);
    let bam = make_bam(&dir);

    // (samtools args, our args) — our -H is samtools' -h.
    let cases: &[(&[&str], &[&str])] = &[
        (&[], &[]),                       // default: header only
        (&["-n", "3"], &["-n", "3"]),     // first 3 records
        (&["-n", "100"], &["-n", "100"]), // more than present
        (&["-h", "2"], &["-H", "2"]),     // first 2 header lines
        (&["-h", "0"], &["-H", "0"]),     // zero header lines
        (&["-h", "1", "-n", "2"], &["-H", "1", "-n", "2"]),
    ];

    for (sa, ou) in cases {
        assert_eq!(
            samtools_head(&bam, sa),
            our_head(&bam, ou),
            "head output differs for samtools {sa:?} / ours {ou:?}"
        );
    }
}
