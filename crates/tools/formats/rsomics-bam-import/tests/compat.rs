//! Field-exact compat against `samtools import` (samtools >= 1.12 when import was added).
//!
//! Verified against samtools 1.23.1 on macOS (aarch64). Each test runs both
//! our binary and samtools import on the same FASTQ fixture, decodes the
//! output with `samtools view`, and checks every SAM field matches.
//!
//! Header comparison is deliberately skipped: samtools adds a @CO "Reverse
//! with: samtools fastq …" line and two @PG lines referencing its command
//! line and the samtools view invocation, while we emit a minimal @HD only.
//! The on-disk read data (QNAME, FLAG, RNAME, POS, MAPQ, CIGAR, RNEXT,
//! PNEXT, TLEN, SEQ, QUAL, aux) must be field-for-field identical.

use std::path::{Path, PathBuf};
use std::process::Command;

fn ours() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-import"))
}

fn golden(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name)
}

/// Version-gate: samtools >= 1.12 (when `samtools import` was introduced).
fn samtools_ready() -> bool {
    let Ok(out) = Command::new("samtools").arg("--version").output() else {
        return false;
    };
    if !out.status.success() {
        return false;
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let version_str = stdout
        .lines()
        .next()
        .unwrap_or("")
        .split_whitespace()
        .nth(1)
        .unwrap_or("");
    let mut parts = version_str.split('.');
    let major: u32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor: u32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    if major > 1 || (major == 1 && minor >= 12) {
        return true;
    }
    eprintln!("SKIP import compat: samtools {version_str} (need >= 1.12)");
    false
}

/// Decode a BAM file to SAM text and return only the alignment lines
/// (everything after the header).
fn bam_to_records(bam: &Path) -> Vec<String> {
    let out = Command::new("samtools")
        .arg("view")
        .arg(bam)
        .output()
        .expect("samtools view failed");
    assert!(
        out.status.success(),
        "samtools view failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.starts_with('@'))
        .map(str::to_owned)
        .collect()
}

fn our_bam(args: &[&str], out: &Path) {
    let status = ours()
        .args(args)
        .arg("-o")
        .arg(out)
        .status()
        .expect("rsomics-bam-import failed to launch");
    assert!(status.success(), "rsomics-bam-import {args:?} failed");
}

fn samtools_bam(args: &[&str], out: &Path) {
    let status = Command::new("samtools")
        .arg("import")
        .args(args)
        .arg("-o")
        .arg(out)
        .status()
        .expect("samtools import failed to launch");
    assert!(status.success(), "samtools import {args:?} failed");
}

#[test]
fn se_matches_samtools() {
    if !samtools_ready() {
        return;
    }
    let dir = std::env::temp_dir().join("rsomics-bam-import-compat-se");
    std::fs::create_dir_all(&dir).unwrap();

    let fq = golden("se.fastq");
    let fq_str = fq.to_str().unwrap();

    our_bam(&["-0", fq_str], &dir.join("ours.bam"));
    samtools_bam(&["-0", fq_str], &dir.join("st.bam"));

    let ours = bam_to_records(&dir.join("ours.bam"));
    let theirs = bam_to_records(&dir.join("st.bam"));

    assert_eq!(ours.len(), theirs.len(), "record count mismatch");
    for (i, (o, t)) in ours.iter().zip(theirs.iter()).enumerate() {
        assert_eq!(o, t, "SE record {i} differs:\n  ours:  {o}\n  theirs:{t}");
    }
}

#[test]
fn pe_separate_matches_samtools() {
    if !samtools_ready() {
        return;
    }
    let dir = std::env::temp_dir().join("rsomics-bam-import-compat-pe");
    std::fs::create_dir_all(&dir).unwrap();

    let r1 = golden("pe_r1.fastq");
    let r2 = golden("pe_r2.fastq");
    let r1s = r1.to_str().unwrap();
    let r2s = r2.to_str().unwrap();

    our_bam(&["-1", r1s, "-2", r2s], &dir.join("ours.bam"));
    samtools_bam(&["-1", r1s, "-2", r2s], &dir.join("st.bam"));

    let ours = bam_to_records(&dir.join("ours.bam"));
    let theirs = bam_to_records(&dir.join("st.bam"));

    assert_eq!(ours.len(), theirs.len(), "record count mismatch");
    for (i, (o, t)) in ours.iter().zip(theirs.iter()).enumerate() {
        assert_eq!(o, t, "PE record {i} differs:\n  ours:  {o}\n  theirs:{t}");
    }
}

#[test]
fn se_with_rg_matches_samtools() {
    if !samtools_ready() {
        return;
    }
    let dir = std::env::temp_dir().join("rsomics-bam-import-compat-rg");
    std::fs::create_dir_all(&dir).unwrap();

    let fq = golden("se.fastq");
    let fq_str = fq.to_str().unwrap();

    our_bam(&["-0", fq_str, "-R", "lib1"], &dir.join("ours.bam"));
    samtools_bam(&["-0", fq_str, "-R", "lib1"], &dir.join("st.bam"));

    let ours = bam_to_records(&dir.join("ours.bam"));
    let theirs = bam_to_records(&dir.join("st.bam"));

    assert_eq!(ours.len(), theirs.len(), "record count mismatch");
    for (i, (o, t)) in ours.iter().zip(theirs.iter()).enumerate() {
        assert_eq!(
            o, t,
            "SE+RG record {i} differs:\n  ours:  {o}\n  theirs:{t}"
        );
    }
}
