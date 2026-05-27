use std::path::{Path, PathBuf};
use std::process::Command;

fn ours() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-consensus"))
}

fn golden_bam() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/reads.bam")
}

/// Returns true when `samtools consensus` subcommand is available at >= 1.23.
/// Older samtools (e.g. CI apt 1.19) lack the subcommand or differ in defaults.
fn samtools_compat_ready() -> bool {
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
    if major == 1 && minor >= 23 {
        return true;
    }
    eprintln!(
        "SKIP consensus compat: samtools {num} (need >= 1.23; consensus subcommand or defaults differ)"
    );
    false
}

fn samtools_consensus(extra: &[&str]) -> Vec<u8> {
    let out = Command::new("samtools")
        .arg("consensus")
        .arg("-m")
        .arg("simple")
        .args(extra)
        .arg(golden_bam())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "samtools consensus {extra:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    out.stdout
}

fn our_consensus(extra: &[&str]) -> Vec<u8> {
    let out = ours()
        .arg("-t1")
        .args(extra)
        .arg(golden_bam())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "ours consensus {extra:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    out.stdout
}

fn assert_byte_exact(label: &str, our_extra: &[&str], their_extra: &[&str]) {
    let theirs = samtools_consensus(their_extra);
    let ours = our_consensus(our_extra);
    if theirs != ours {
        let t = String::from_utf8_lossy(&theirs);
        let o = String::from_utf8_lossy(&ours);
        for (i, (tl, ol)) in t.lines().zip(o.lines()).enumerate() {
            if tl != ol {
                panic!("[{label}] line {i} differs:\n  samtools: {tl}\n  ours:     {ol}");
            }
        }
        panic!(
            "[{label}] output differs in length: samtools={} ours={} lines",
            t.lines().count(),
            o.lines().count()
        );
    }
}

/// Default simple-mode FASTA consensus: no quality weights, call-fract 0.75,
/// het-fract 0.5, min-depth 1, line-len 70.
#[test]
fn consensus_default_simple() {
    if !samtools_compat_ready() {
        return;
    }
    // samtools default is bayesian; we must force simple mode on their side.
    // Our binary defaults to simple, so no extra flags needed.
    assert_byte_exact("default-simple", &[], &[]);
}

/// Quality-weighted simple consensus (`--use-qual`).
#[test]
fn consensus_use_qual() {
    if !samtools_compat_ready() {
        return;
    }
    assert_byte_exact("use-qual", &["--use-qual"], &["--use-qual"]);
}

/// Higher min-depth gate.
#[test]
fn consensus_min_depth() {
    if !samtools_compat_ready() {
        return;
    }
    assert_byte_exact("min-depth-5", &["-d", "5"], &["-d", "5"]);
}

/// Line-len wrapping check.
#[test]
fn consensus_line_len() {
    if !samtools_compat_ready() {
        return;
    }
    assert_byte_exact("line-len-60", &["-l", "60"], &["-l", "60"]);
}

/// Binary exits zero on a valid BAM.
#[test]
fn exit_zero_on_valid() {
    let out = ours().arg("-t1").arg(golden_bam()).output().unwrap();
    assert!(
        out.status.success(),
        "expected exit 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Binary exits non-zero when given a non-existent file.
#[test]
fn exit_nonzero_on_missing_file() {
    let out = ours().arg("/nonexistent/file.bam").output().unwrap();
    assert!(
        !out.status.success(),
        "expected non-zero exit for missing file"
    );
}
