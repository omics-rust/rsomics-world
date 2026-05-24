//! Byte-exact compatibility tests against `vsearch --derep_fulllength`.
//!
//! Requires `vsearch` on PATH.  If absent, tests are skipped with a clear
//! message (no silent pass).

use std::io::{BufWriter, Write};
use std::process::Command;

fn vsearch_on_path() -> bool {
    Command::new("vsearch").arg("--version").output().is_ok()
}

fn xshift(s: &mut u64) -> u64 {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    *s
}

fn golden(name: &str) -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/golden");
    p.push(name);
    p
}

fn derep_binary() -> std::path::PathBuf {
    let exe = env!("CARGO_BIN_EXE_rsomics-derep");
    exe.into()
}

fn run_vsearch(input: &std::path::Path, output: &std::path::Path, sizein: bool) {
    let mut cmd = Command::new("vsearch");
    cmd.arg("--derep_fulllength")
        .arg(input)
        .arg("--output")
        .arg(output)
        .arg("--sizeout");
    if sizein {
        cmd.arg("--sizein");
    }
    let status = cmd.status().expect("vsearch failed to run");
    assert!(status.success(), "vsearch exited non-zero");
}

fn run_ours(input: &std::path::Path, output: &std::path::Path, sizein: bool) {
    let mut cmd = Command::new(derep_binary());
    cmd.arg(input).arg("-o").arg(output).arg("-q");
    if sizein {
        cmd.arg("--sizein");
    }
    let status = cmd.status().expect("rsomics-derep failed to run");
    assert!(status.success(), "rsomics-derep exited non-zero");
}

// Deterministic pseudo-random: xorshift64.
fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
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

    run_vsearch(&input, vsearch_out.path(), false);
    run_ours(&input, ours_out.path(), false);

    let expected = std::fs::read(vsearch_out.path()).unwrap();
    let actual = std::fs::read(ours_out.path()).unwrap();
    assert_eq!(
        actual,
        expected,
        "basic.fasta: output differs from vsearch\n\
         ours:\n{}\nvsearch:\n{}",
        String::from_utf8_lossy(&actual),
        String::from_utf8_lossy(&expected)
    );
}

#[test]
fn compat_sizein_byte_exact() {
    if !vsearch_on_path() {
        eprintln!("SKIP: vsearch not on PATH");
        return;
    }
    let input = golden("sizein.fasta");
    let vsearch_out = tempfile::NamedTempFile::new().unwrap();
    let ours_out = tempfile::NamedTempFile::new().unwrap();

    run_vsearch(&input, vsearch_out.path(), true);
    run_ours(&input, ours_out.path(), true);

    let expected = std::fs::read(vsearch_out.path()).unwrap();
    let actual = std::fs::read(ours_out.path()).unwrap();
    assert_eq!(
        actual,
        expected,
        "sizein.fasta: output differs from vsearch\n\
         ours:\n{}\nvsearch:\n{}",
        String::from_utf8_lossy(&actual),
        String::from_utf8_lossy(&expected)
    );
}

#[test]
fn compat_large_synthetic_byte_exact() {
    if !vsearch_on_path() {
        eprintln!("SKIP: vsearch not on PATH");
        return;
    }

    // 10 distinct sequences × 100 repetitions = 1000 seqs total.
    let bases = b"ACGT";
    let seq_len = 200usize;
    let num_distinct = 10usize;
    let repeats = 100usize;
    let base_state: u64 = 0xDEAD_BEEF_1234_5678;

    let distinct_seqs: Vec<Vec<u8>> = (0..num_distinct)
        .map(|d| {
            let mut seed = base_state
                .wrapping_add(d as u64)
                .wrapping_mul(0x9E37_79B9_7F4A_7C15);
            (0..seq_len)
                .map(|_| bases[(xorshift(&mut seed) % 4) as usize])
                .collect()
        })
        .collect();

    let tmp_in = tempfile::NamedTempFile::new().unwrap();
    let mut f = BufWriter::new(std::fs::File::create(tmp_in.path()).unwrap());

    for (d, seq) in distinct_seqs.iter().enumerate() {
        writeln!(f, ">distinct_{d}").unwrap();
        f.write_all(seq.as_slice()).unwrap();
        writeln!(f).unwrap();
    }
    // Additional copies with different header names.
    let mut seq_idx: u64 = 0;
    for rep in 1..repeats {
        for orig_seq in &distinct_seqs {
            seq_idx += 1;
            writeln!(f, ">seq_{seq_idx}").unwrap();
            // Occasionally use lowercase to exercise normalisation.
            let seq: Vec<u8> = if rep % 5 == 0 {
                orig_seq.iter().map(|&b| b.to_ascii_lowercase()).collect()
            } else {
                orig_seq.clone()
            };
            f.write_all(&seq).unwrap();
            writeln!(f).unwrap();
        }
    }
    drop(f);

    let vsearch_out = tempfile::NamedTempFile::new().unwrap();
    let ours_out = tempfile::NamedTempFile::new().unwrap();

    run_vsearch(tmp_in.path(), vsearch_out.path(), false);
    run_ours(tmp_in.path(), ours_out.path(), false);

    let expected = std::fs::read(vsearch_out.path()).unwrap();
    let actual = std::fs::read(ours_out.path()).unwrap();
    assert_eq!(
        actual,
        expected,
        "large synthetic (1000 seqs, 10 distinct): output differs from vsearch\n\
         ours:\n{}\nvsearch:\n{}",
        String::from_utf8_lossy(&actual),
        String::from_utf8_lossy(&expected)
    );
}

#[test]
fn compat_case_preservation_and_minseqlength() {
    if !vsearch_on_path() {
        eprintln!("SKIP: vsearch not on PATH");
        return;
    }
    // case_and_short.fasta contains:
    //   - lowercase-first-occurrence duplicates (case preservation)
    //   - a sequence < 32 nt (should be discarded by minseqlength 32)
    //   - U-containing RNA sequence (U preserved in output, T-equivalent for matching)
    let input = golden("case_and_short.fasta");
    let vsearch_out = tempfile::NamedTempFile::new().unwrap();
    let ours_out = tempfile::NamedTempFile::new().unwrap();

    run_vsearch(&input, vsearch_out.path(), false);
    run_ours(&input, ours_out.path(), false);

    let expected = std::fs::read(vsearch_out.path()).unwrap();
    let actual = std::fs::read(ours_out.path()).unwrap();
    assert_eq!(
        actual,
        expected,
        "case_and_short.fasta: output differs from vsearch\n\
         ours:\n{}\nvsearch:\n{}",
        String::from_utf8_lossy(&actual),
        String::from_utf8_lossy(&expected)
    );
}

#[test]
fn compat_comprehensive_synthetic_byte_exact() {
    if !vsearch_on_path() {
        eprintln!("SKIP: vsearch not on PATH");
        return;
    }

    // Comprehensive synthetic: lowercase firsts, uppercase dups, U-containing,
    // short sequences (<32), existing ;size= annotations, abundance ties.
    let tmp_in = tempfile::NamedTempFile::new().unwrap();
    let mut f = BufWriter::new(std::fs::File::create(tmp_in.path()).unwrap());
    let bases = b"ACGT";
    let state: u64 = 0xFEED_BEEF_CAFE_1234;

    // 8 distinct 60-bp sequences
    let distinct: Vec<Vec<u8>> = (0..8u64)
        .map(|d| {
            let mut seed = state.wrapping_add(d).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            (0..60usize)
                .map(|_| bases[(xshift(&mut seed) % 4) as usize])
                .collect()
        })
        .collect();

    // First occurrence: lowercase
    for (i, seq) in distinct.iter().enumerate() {
        let lower: Vec<u8> = seq.iter().map(|&b| b.to_ascii_lowercase()).collect();
        writeln!(f, ">first_{i}").unwrap();
        f.write_all(&lower).unwrap();
        writeln!(f).unwrap();
    }
    // Some with U substitution (same as T for matching)
    for (i, seq) in distinct[..4].iter().enumerate() {
        let u_seq: Vec<u8> = seq
            .iter()
            .map(|&b| if b == b'T' { b'U' } else { b })
            .collect();
        writeln!(f, ">u_dup_{i}").unwrap();
        f.write_all(&u_seq).unwrap();
        writeln!(f).unwrap();
    }
    // Uppercase duplicates
    for (i, seq) in distinct.iter().enumerate() {
        writeln!(f, ">upper_dup_{i}").unwrap();
        f.write_all(seq).unwrap();
        writeln!(f).unwrap();
    }
    // Short sequences (< 32 nt) — should be discarded
    for i in 0..3u32 {
        writeln!(f, ">short_{i}").unwrap();
        writeln!(f, "ACGTACGT").unwrap();
    }
    // Sequences with existing ;size= annotation
    writeln!(f, ">annotated;size=3").unwrap();
    f.write_all(&distinct[0]).unwrap();
    writeln!(f).unwrap();

    drop(f);

    let vsearch_out = tempfile::NamedTempFile::new().unwrap();
    let ours_out = tempfile::NamedTempFile::new().unwrap();

    run_vsearch(tmp_in.path(), vsearch_out.path(), false);
    run_ours(tmp_in.path(), ours_out.path(), false);

    let expected = std::fs::read(vsearch_out.path()).unwrap();
    let actual = std::fs::read(ours_out.path()).unwrap();
    assert_eq!(
        actual,
        expected,
        "comprehensive synthetic: output differs from vsearch\n\
         ours:\n{}\nvsearch:\n{}",
        String::from_utf8_lossy(&actual),
        String::from_utf8_lossy(&expected)
    );
}

/// Heavy-ties tie-break order test.
///
/// ~12 000 distinct sequences with sizes 1-4 (many equal-abundance groups).
/// vsearch breaks ties by byte-wise strcmp on the **original input header**
/// (with `;size=N` annotation intact), not the stripped label.  This test
/// catches regressions in that ordering.
#[test]
fn compat_heavy_ties_order_byte_exact() {
    if !vsearch_on_path() {
        eprintln!("SKIP: vsearch not on PATH");
        return;
    }

    let bases = b"ACGT";
    let seq_len = 60usize;
    let num_distinct = 12_000usize;

    let tmp_in = tempfile::NamedTempFile::new().unwrap();
    let mut f = BufWriter::new(std::fs::File::create(tmp_in.path()).unwrap());

    let mut seed: u64 = 0x4142_4344_4546_4748;
    for i in 0..num_distinct {
        // size 1-4, cycling deterministically
        let size = (i % 4) + 1;
        let seq: Vec<u8> = (0..seq_len)
            .map(|_| bases[(xorshift(&mut seed) % 4) as usize])
            .collect();
        writeln!(f, ">s{};size={}", i + 1, size).unwrap();
        f.write_all(&seq).unwrap();
        writeln!(f).unwrap();
    }
    drop(f);

    let vsearch_out = tempfile::NamedTempFile::new().unwrap();
    let ours_out = tempfile::NamedTempFile::new().unwrap();

    run_vsearch(tmp_in.path(), vsearch_out.path(), true);
    run_ours(tmp_in.path(), ours_out.path(), true);

    let expected = std::fs::read(vsearch_out.path()).unwrap();
    let actual = std::fs::read(ours_out.path()).unwrap();
    assert_eq!(
        actual,
        expected,
        "heavy-ties (12k distinct, sizes 1-4, sizein): output differs from vsearch\n\
         ours:\n{}\nvsearch:\n{}",
        String::from_utf8_lossy(&actual),
        String::from_utf8_lossy(&expected)
    );
}
