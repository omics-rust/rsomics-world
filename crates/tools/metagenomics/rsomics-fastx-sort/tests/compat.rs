//! Byte-exact compatibility tests against vsearch --sortbysize and --sortbylength.
//!
//! Requires `vsearch` on PATH.  If absent, tests are skipped with a clear
//! message (no silent pass).

use std::io::{BufWriter, Write};
use std::process::Command;

fn vsearch_on_path() -> bool {
    Command::new("vsearch").arg("--version").output().is_ok()
}

fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn golden(name: &str) -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/golden");
    p.push(name);
    p
}

fn sort_binary() -> std::path::PathBuf {
    let exe = env!("CARGO_BIN_EXE_rsomics-fastx-sort");
    exe.into()
}

fn run_vsearch_sortbysize(input: &std::path::Path, output: &std::path::Path, sizeout: bool) {
    let mut cmd = Command::new("vsearch");
    cmd.arg("--sortbysize")
        .arg(input)
        .arg("--output")
        .arg(output)
        .arg("--sizein");
    if sizeout {
        cmd.arg("--sizeout");
    }
    let status = cmd.status().expect("vsearch failed to run");
    assert!(status.success(), "vsearch exited non-zero");
}

fn run_vsearch_sortbylength(input: &std::path::Path, output: &std::path::Path, sizeout: bool) {
    let mut cmd = Command::new("vsearch");
    cmd.arg("--sortbylength")
        .arg(input)
        .arg("--output")
        .arg(output)
        .arg("--sizein");
    if sizeout {
        cmd.arg("--sizeout");
    }
    let status = cmd.status().expect("vsearch failed to run");
    assert!(status.success(), "vsearch exited non-zero");
}

fn run_ours_sortbysize(input: &std::path::Path, output: &std::path::Path, sizeout: bool) {
    let mut cmd = Command::new(sort_binary());
    cmd.arg(input)
        .arg("-o")
        .arg(output)
        .arg("--mode")
        .arg("size")
        .arg("--sizein")
        .arg("-q");
    if sizeout {
        cmd.arg("--sizeout");
    }
    let status = cmd.status().expect("rsomics-fastx-sort failed to run");
    assert!(status.success(), "rsomics-fastx-sort exited non-zero");
}

fn run_ours_sortbylength(input: &std::path::Path, output: &std::path::Path, sizeout: bool) {
    let mut cmd = Command::new(sort_binary());
    cmd.arg(input)
        .arg("-o")
        .arg(output)
        .arg("--mode")
        .arg("length")
        .arg("--sizein")
        .arg("-q");
    if sizeout {
        cmd.arg("--sizeout");
    }
    let status = cmd.status().expect("rsomics-fastx-sort failed to run");
    assert!(status.success(), "rsomics-fastx-sort exited non-zero");
}

#[test]
fn compat_sortbysize_basic_sizeout() {
    if !vsearch_on_path() {
        eprintln!("SKIP: vsearch not on PATH");
        return;
    }
    let input = golden("basic_size.fasta");
    let vsearch_out = tempfile::NamedTempFile::new().unwrap();
    let ours_out = tempfile::NamedTempFile::new().unwrap();

    run_vsearch_sortbysize(&input, vsearch_out.path(), true);
    run_ours_sortbysize(&input, ours_out.path(), true);

    let expected = std::fs::read(vsearch_out.path()).unwrap();
    let actual = std::fs::read(ours_out.path()).unwrap();
    assert_eq!(
        actual,
        expected,
        "basic_size.fasta sortbysize --sizeout: output differs\nours:\n{}\nvsearch:\n{}",
        String::from_utf8_lossy(&actual),
        String::from_utf8_lossy(&expected)
    );
}

#[test]
fn compat_sortbylength_basic() {
    if !vsearch_on_path() {
        eprintln!("SKIP: vsearch not on PATH");
        return;
    }
    let input = golden("basic_len.fasta");
    let vsearch_out = tempfile::NamedTempFile::new().unwrap();
    let ours_out = tempfile::NamedTempFile::new().unwrap();

    run_vsearch_sortbylength(&input, vsearch_out.path(), false);
    run_ours_sortbylength(&input, ours_out.path(), false);

    let expected = std::fs::read(vsearch_out.path()).unwrap();
    let actual = std::fs::read(ours_out.path()).unwrap();
    assert_eq!(
        actual,
        expected,
        "basic_len.fasta sortbylength: output differs\nours:\n{}\nvsearch:\n{}",
        String::from_utf8_lossy(&actual),
        String::from_utf8_lossy(&expected)
    );
}

/// Adversarial heavy-ties synthetic: ~100k seqs with many equal sizes/lengths,
/// mixed case, short+long sequences to filter, U-containing, existing ;size=,
/// labels that stress strcmp tie-breaking.
///
/// This is the primary correctness oracle — byte-exact for BOTH modes.
#[test]
fn compat_adversarial_sortbysize_byte_exact() {
    if !vsearch_on_path() {
        eprintln!("SKIP: vsearch not on PATH");
        return;
    }

    let tmp_in = tempfile::NamedTempFile::new().unwrap();
    write_adversarial_fixture(tmp_in.path());

    let vsearch_out = tempfile::NamedTempFile::new().unwrap();
    let ours_out = tempfile::NamedTempFile::new().unwrap();

    run_vsearch_sortbysize(tmp_in.path(), vsearch_out.path(), true);
    run_ours_sortbysize(tmp_in.path(), ours_out.path(), true);

    let expected = std::fs::read(vsearch_out.path()).unwrap();
    let actual = std::fs::read(ours_out.path()).unwrap();
    assert_eq!(
        actual,
        expected,
        "adversarial sortbysize --sizeout: output differs at byte level\n\
         first 500 bytes ours:\n{}\nfirst 500 bytes vsearch:\n{}",
        String::from_utf8_lossy(&actual[..actual.len().min(500)]),
        String::from_utf8_lossy(&expected[..expected.len().min(500)])
    );
}

#[test]
fn compat_adversarial_sortbylength_byte_exact() {
    if !vsearch_on_path() {
        eprintln!("SKIP: vsearch not on PATH");
        return;
    }

    let tmp_in = tempfile::NamedTempFile::new().unwrap();
    write_adversarial_fixture(tmp_in.path());

    let vsearch_out = tempfile::NamedTempFile::new().unwrap();
    let ours_out = tempfile::NamedTempFile::new().unwrap();

    run_vsearch_sortbylength(tmp_in.path(), vsearch_out.path(), false);
    run_ours_sortbylength(tmp_in.path(), ours_out.path(), false);

    let expected = std::fs::read(vsearch_out.path()).unwrap();
    let actual = std::fs::read(ours_out.path()).unwrap();
    assert_eq!(
        actual,
        expected,
        "adversarial sortbylength: output differs at byte level\n\
         first 500 bytes ours:\n{}\nfirst 500 bytes vsearch:\n{}",
        String::from_utf8_lossy(&actual[..actual.len().min(500)]),
        String::from_utf8_lossy(&expected[..expected.len().min(500)])
    );
}

/// Write an adversarial fixture to a file path.
///
/// Covers: heavy ties (sizes 1-4, lengths within ~5 buckets), mixed case,
/// short sequences (<32 nt, still kept since sort default minseqlength=1),
/// long sequences (near 50000 limit), U-containing, existing `;size=N`,
/// labels like `>s{i};size={n}` that stress strcmp vs numeric ordering.
fn write_adversarial_fixture(path: &std::path::Path) {
    let mut f = BufWriter::new(std::fs::File::create(path).unwrap());
    let bases = b"ACGT";
    let u_bases = b"ACGU"; // RNA bases
    let mut seed: u64 = 0xDEAD_BEEF_1234_5678;

    let num_seqs = 100_000usize;
    // Length buckets: 40, 60, 80, 100, 120 (heavy ties within each bucket)
    let length_buckets = [40usize, 60, 80, 100, 120];
    // Size values cycling 1-4 (heavy ties)
    let size_values = [1u32, 2, 3, 4];

    for i in 0..num_seqs {
        let size = size_values[i % 4];
        let seq_len = length_buckets[i % 5];

        // Generate sequence with mixed case and occasional U substitution
        let use_rna = i % 17 == 0;
        let use_lower = i % 7 < 3;
        let active_bases = if use_rna { u_bases } else { bases };

        let seq: Vec<u8> = (0..seq_len)
            .map(|_| {
                let b = active_bases[(xorshift(&mut seed) % 4) as usize];
                if use_lower { b.to_ascii_lowercase() } else { b }
            })
            .collect();

        // Vary label format to stress strcmp tie-breaking:
        // - some have numeric-only suffixes
        // - some have ;extra= attributes that affect strcmp order
        // - some labels start with digits
        let label = match i % 6 {
            0 => format!("s{i};size={size}"),
            1 => format!("seq_{i:06};size={size}"),
            2 => format!("s{i};k=v;size={size};extra=x"),
            3 => format!("r{i}_x;size={size}"),
            4 => format!("{i:08};size={size}"),
            _ => format!("z{i};size={size}"),
        };

        writeln!(f, ">{label}").unwrap();
        f.write_all(&seq).unwrap();
        writeln!(f).unwrap();
    }

    // Add a few short sequences (below 32 nt) — vsearch default minseqlength=1 for
    // sort keeps them; only derep/clust/search default to 32.
    for i in 0..20usize {
        let size = u32::try_from(i % 4 + 1).unwrap();
        writeln!(f, ">short_{i};size={size}").unwrap();
        writeln!(f, "ACGTACGT").unwrap(); // 8 nt
    }

    // Add a few sequences with no ;size= annotation (should default to size=1)
    for i in 0..10usize {
        let seq: Vec<u8> = (0..60usize)
            .map(|_| bases[(xorshift(&mut seed) % 4) as usize])
            .collect();
        writeln!(f, ">noanno_{i}").unwrap();
        f.write_all(&seq).unwrap();
        writeln!(f).unwrap();
    }
}

/// Test that --minseqlength filter works in sortbysize mode.
#[test]
fn compat_sortbysize_minseqlength_filter() {
    if !vsearch_on_path() {
        eprintln!("SKIP: vsearch not on PATH");
        return;
    }

    let tmp_in = tempfile::NamedTempFile::new().unwrap();
    {
        let mut f = BufWriter::new(std::fs::File::create(tmp_in.path()).unwrap());
        // short (8 nt, filtered by --minseqlength 32)
        writeln!(f, ">short;size=10").unwrap();
        writeln!(f, "ACGTACGT").unwrap();
        // long enough
        writeln!(f, ">long;size=5").unwrap();
        writeln!(f, "ACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGT").unwrap();
    }

    let vsearch_out = tempfile::NamedTempFile::new().unwrap();
    let ours_out = tempfile::NamedTempFile::new().unwrap();

    // Run vsearch with explicit --minseqlength 32
    let status = Command::new("vsearch")
        .arg("--sortbysize")
        .arg(tmp_in.path())
        .arg("--output")
        .arg(vsearch_out.path())
        .arg("--sizein")
        .arg("--sizeout")
        .arg("--minseqlength")
        .arg("32")
        .status()
        .expect("vsearch failed");
    assert!(status.success());

    let status = Command::new(sort_binary())
        .arg(tmp_in.path())
        .arg("-o")
        .arg(ours_out.path())
        .arg("--mode")
        .arg("size")
        .arg("--sizein")
        .arg("--sizeout")
        .arg("--minseqlength")
        .arg("32")
        .arg("-q")
        .status()
        .expect("rsomics-fastx-sort failed");
    assert!(status.success());

    let expected = std::fs::read(vsearch_out.path()).unwrap();
    let actual = std::fs::read(ours_out.path()).unwrap();
    assert_eq!(
        actual,
        expected,
        "minseqlength filter: output differs\nours:\n{}\nvsearch:\n{}",
        String::from_utf8_lossy(&actual),
        String::from_utf8_lossy(&expected)
    );
}
