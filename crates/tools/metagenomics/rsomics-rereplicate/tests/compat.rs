//! Byte-exact compatibility tests against `vsearch --rereplicate`.
//!
//! Requires `vsearch` on PATH.  If absent, tests are skipped.

use std::io::{BufWriter, Write};
use std::process::Command;

fn vsearch_on_path() -> bool {
    Command::new("vsearch").arg("--version").output().is_ok()
}

fn golden(name: &str) -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/golden");
    p.push(name);
    p
}

fn rerep_binary() -> std::path::PathBuf {
    env!("CARGO_BIN_EXE_rsomics-rereplicate").into()
}

fn run_vsearch(input: &std::path::Path, output: &std::path::Path) {
    let status = Command::new("vsearch")
        .arg("--rereplicate")
        .arg(input)
        .arg("--output")
        .arg(output)
        .arg("--quiet")
        .status()
        .expect("vsearch failed to run");
    assert!(status.success(), "vsearch exited non-zero");
}

fn run_ours(input: &std::path::Path, output: &std::path::Path) {
    let status = Command::new(rerep_binary())
        .arg(input)
        .arg("-o")
        .arg(output)
        .arg("-q")
        .status()
        .expect("rsomics-rereplicate failed to run");
    assert!(status.success(), "rsomics-rereplicate exited non-zero");
}

fn run_vsearch_sizeout(input: &std::path::Path, output: &std::path::Path) {
    let status = Command::new("vsearch")
        .arg("--rereplicate")
        .arg(input)
        .arg("--output")
        .arg(output)
        .arg("--sizeout")
        .arg("--quiet")
        .status()
        .expect("vsearch failed to run");
    assert!(status.success(), "vsearch exited non-zero");
}

fn run_ours_sizeout(input: &std::path::Path, output: &std::path::Path) {
    let status = Command::new(rerep_binary())
        .arg(input)
        .arg("-o")
        .arg(output)
        .arg("--sizeout")
        .arg("-q")
        .status()
        .expect("rsomics-rereplicate failed to run");
    assert!(status.success(), "rsomics-rereplicate exited non-zero");
}

#[test]
fn compat_basic_byte_exact() {
    if !vsearch_on_path() {
        eprintln!("SKIP: vsearch not on PATH");
        return;
    }
    let input = golden("basic.fasta");
    let vsearch_out = tempfile::NamedTempFile::new().unwrap();
    let ours_out = tempfile::NamedTempFile::new().unwrap();
    run_vsearch(&input, vsearch_out.path());
    run_ours(&input, ours_out.path());
    let expected = std::fs::read(vsearch_out.path()).unwrap();
    let actual = std::fs::read(ours_out.path()).unwrap();
    assert_eq!(
        actual,
        expected,
        "basic.fasta: output differs from vsearch\nours:\n{}\nvsearch:\n{}",
        String::from_utf8_lossy(&actual),
        String::from_utf8_lossy(&expected)
    );
}

#[test]
fn compat_adversarial_byte_exact() {
    if !vsearch_on_path() {
        eprintln!("SKIP: vsearch not on PATH");
        return;
    }
    let input = golden("adversarial.fasta");
    let vsearch_out = tempfile::NamedTempFile::new().unwrap();
    let ours_out = tempfile::NamedTempFile::new().unwrap();
    run_vsearch(&input, vsearch_out.path());
    run_ours(&input, ours_out.path());
    let expected = std::fs::read(vsearch_out.path()).unwrap();
    let actual = std::fs::read(ours_out.path()).unwrap();
    assert_eq!(
        actual,
        expected,
        "adversarial.fasta: output differs from vsearch\nours:\n{}\nvsearch:\n{}",
        String::from_utf8_lossy(&actual),
        String::from_utf8_lossy(&expected)
    );
}

#[test]
fn compat_sizeout_byte_exact() {
    if !vsearch_on_path() {
        eprintln!("SKIP: vsearch not on PATH");
        return;
    }
    let input = golden("adversarial.fasta");
    let vsearch_out = tempfile::NamedTempFile::new().unwrap();
    let ours_out = tempfile::NamedTempFile::new().unwrap();
    run_vsearch_sizeout(&input, vsearch_out.path());
    run_ours_sizeout(&input, ours_out.path());
    let expected = std::fs::read(vsearch_out.path()).unwrap();
    let actual = std::fs::read(ours_out.path()).unwrap();
    assert_eq!(
        actual,
        expected,
        "adversarial.fasta (--sizeout): output differs from vsearch\nours:\n{}\nvsearch:\n{}",
        String::from_utf8_lossy(&actual),
        String::from_utf8_lossy(&expected)
    );
}

fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

#[test]
fn compat_large_synthetic_byte_exact() {
    if !vsearch_on_path() {
        eprintln!("SKIP: vsearch not on PATH");
        return;
    }

    // Build a large synthetic: 500 distinct 120-bp sequences, sizes 1-10.
    // Total reads: sum(1..=10) * 50 = 2750 from 500 amplicons.
    let bases = b"ACGT";
    let tmp_in = tempfile::NamedTempFile::new().unwrap();
    let mut f = BufWriter::new(std::fs::File::create(tmp_in.path()).unwrap());
    let mut seed: u64 = 0xDEAD_BEEF_4242_1234;

    for i in 0u64..500 {
        let size = (i % 10) + 1;
        let seq: Vec<u8> = (0..120)
            .map(|_| bases[(xorshift(&mut seed) % 4) as usize])
            .collect();
        // Some with mixed case
        let seq: Vec<u8> = if i % 7 == 0 {
            seq.iter().map(|&b| b.to_ascii_lowercase()).collect()
        } else {
            seq
        };
        // Some with U
        let seq: Vec<u8> = if i % 13 == 0 {
            seq.iter()
                .map(|&b| {
                    if b == b't' || b == b'T' {
                        if i % 2 == 0 { b'U' } else { b'u' }
                    } else {
                        b
                    }
                })
                .collect()
        } else {
            seq
        };
        writeln!(f, ">amplicon_{i};size={size}").unwrap();
        f.write_all(&seq).unwrap();
        writeln!(f).unwrap();
    }
    drop(f);

    let vsearch_out = tempfile::NamedTempFile::new().unwrap();
    let ours_out = tempfile::NamedTempFile::new().unwrap();
    run_vsearch(tmp_in.path(), vsearch_out.path());
    run_ours(tmp_in.path(), ours_out.path());

    let expected = std::fs::read(vsearch_out.path()).unwrap();
    let actual = std::fs::read(ours_out.path()).unwrap();
    assert_eq!(
        actual, expected,
        "large synthetic (500 amplicons): output differs from vsearch"
    );
}
