use std::path::{Path, PathBuf};
use std::process::Command;

fn ours() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-mpileup"))
}

fn golden_bam() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/coord_sorted.bam")
}

fn golden_ref() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/ref.fa")
}

/// mpileup's byte-exact compat is version-pinned to samtools >= 1.23. The pileup
/// text format (overlap-removal selection, BAQ defaults, indel/`*` encoding) has
/// shifted across releases, and CI's apt samtools (1.19.2) emits different output
/// — so gate on version to avoid a false-fail on a mismatched CI samtools, per
/// the fastp-compat-version precedent.
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
        "SKIP mpileup compat: samtools {num} (need >= 1.23; pileup output differs on older releases)"
    );
    false
}

/// Run `samtools mpileup <extra_args> <bam>` and capture stdout bytes.
fn samtools_pileup(extra: &[&str]) -> Vec<u8> {
    let out = Command::new("samtools")
        .arg("mpileup")
        .args(extra)
        .arg(golden_bam())
        .output()
        .unwrap();
    assert!(out.status.success(), "samtools mpileup {extra:?} failed");
    out.stdout
}

/// Run ours `-t1 <extra_args> <bam>` and capture stdout bytes.
fn our_pileup(extra: &[&str]) -> Vec<u8> {
    let out = ours()
        .arg("-t1")
        .args(extra)
        .arg(golden_bam())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "ours mpileup {extra:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    out.stdout
}

fn assert_byte_exact(label: &str, extra: &[&str]) {
    let theirs = samtools_pileup(extra);
    let ours = our_pileup(extra);
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

/// No-reference default pileup: ref column is `N`, bases are literal letters.
/// Exercises overlap removal, orphan filter, head/tail markers and indel
/// encoding (the golden fixture has overlapping proper pairs and an insertion +
/// deletion).
#[test]
fn mpileup_default_no_ref() {
    if !samtools_compat_ready() {
        return;
    }
    assert_byte_exact("no-ref default", &[]);
}

#[test]
fn mpileup_no_ref_flag_variants() {
    if !samtools_compat_ready() {
        return;
    }
    assert_byte_exact("min-BQ 20", &["-Q", "20"]);
    assert_byte_exact("min-BQ 0", &["-Q", "0"]);
    assert_byte_exact("count-orphans", &["-A"]);
    assert_byte_exact("ignore-overlaps", &["-x"]);
    assert_byte_exact("all positions", &["-a"]);
    assert_byte_exact("all positions all refs", &["-aa"]);
}

/// Reference-aware pileup with BAQ disabled (`-f ref -B`): ref-matching bases
/// render as `.`/`,`, deletions show the reference bases. BAQ (the samtools
/// default for `-f`) is not implemented, so `-B` is required for byte-exactness.
#[test]
fn mpileup_ref_no_baq() {
    if !samtools_compat_ready() {
        return;
    }
    let r = golden_ref();
    let rs = r.to_str().unwrap();
    assert_byte_exact("ref -B", &["-f", rs, "-B"]);
    assert_byte_exact("ref -B -a", &["-f", rs, "-B", "-a"]);
    assert_byte_exact("ref -B -aa", &["-f", rs, "-B", "-aa"]);
    assert_byte_exact("ref -B -Q 20", &["-f", rs, "-B", "-Q", "20"]);
}
