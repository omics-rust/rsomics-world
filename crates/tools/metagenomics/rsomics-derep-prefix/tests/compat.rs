//! Byte-exact compatibility tests against `vsearch --derep_prefix`.
//!
//! Requires `vsearch` on PATH. If absent, tests are skipped with a clear
//! message (no silent pass).

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

fn derep_prefix_binary() -> std::path::PathBuf {
    env!("CARGO_BIN_EXE_rsomics-derep-prefix").into()
}

fn run_vsearch(input: &std::path::Path, output: &std::path::Path, sizein: bool) {
    let mut cmd = Command::new("vsearch");
    cmd.arg("--derep_prefix")
        .arg(input)
        .arg("--output")
        .arg(output)
        .arg("--sizeout")
        .arg("--quiet");
    if sizein {
        cmd.arg("--sizein");
    }
    let status = cmd.status().expect("vsearch failed to run");
    assert!(status.success(), "vsearch exited non-zero");
}

fn run_ours(input: &std::path::Path, output: &std::path::Path, sizein: bool) {
    let mut cmd = Command::new(derep_prefix_binary());
    cmd.arg(input).arg("-o").arg(output).arg("-q");
    if sizein {
        cmd.arg("--sizein");
    }
    let status = cmd.status().expect("rsomics-derep-prefix failed to run");
    assert!(status.success(), "rsomics-derep-prefix exited non-zero");
}

fn xorshift(s: &mut u64) -> u64 {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    *s
}

#[test]
fn compat_golden_prefix_basic() {
    if !vsearch_on_path() {
        eprintln!("SKIP: vsearch not on PATH");
        return;
    }
    let input = golden("prefix_basic.fasta");
    let vs_out = tempfile::NamedTempFile::new().unwrap();
    let our_out = tempfile::NamedTempFile::new().unwrap();

    run_vsearch(&input, vs_out.path(), false);
    run_ours(&input, our_out.path(), false);

    let expected = std::fs::read(vs_out.path()).unwrap();
    let actual = std::fs::read(our_out.path()).unwrap();
    assert_eq!(
        actual,
        expected,
        "prefix_basic.fasta: output differs from vsearch\n\
         ours:\n{}\nvsearch:\n{}",
        String::from_utf8_lossy(&actual),
        String::from_utf8_lossy(&expected)
    );
}

#[test]
fn compat_golden_case_u() {
    if !vsearch_on_path() {
        eprintln!("SKIP: vsearch not on PATH");
        return;
    }
    let input = golden("case_u_prefix.fasta");
    let vs_out = tempfile::NamedTempFile::new().unwrap();
    let our_out = tempfile::NamedTempFile::new().unwrap();

    run_vsearch(&input, vs_out.path(), false);
    run_ours(&input, our_out.path(), false);

    let expected = std::fs::read(vs_out.path()).unwrap();
    let actual = std::fs::read(our_out.path()).unwrap();
    assert_eq!(
        actual,
        expected,
        "case_u_prefix.fasta: output differs from vsearch\n\
         ours:\n{}\nvsearch:\n{}",
        String::from_utf8_lossy(&actual),
        String::from_utf8_lossy(&expected)
    );
}

#[test]
fn compat_golden_sizein() {
    if !vsearch_on_path() {
        eprintln!("SKIP: vsearch not on PATH");
        return;
    }
    let input = golden("sizein_prefix.fasta");
    let vs_out = tempfile::NamedTempFile::new().unwrap();
    let our_out = tempfile::NamedTempFile::new().unwrap();

    run_vsearch(&input, vs_out.path(), true);
    run_ours(&input, our_out.path(), true);

    let expected = std::fs::read(vs_out.path()).unwrap();
    let actual = std::fs::read(our_out.path()).unwrap();
    assert_eq!(
        actual,
        expected,
        "sizein_prefix.fasta: output differs from vsearch\n\
         ours:\n{}\nvsearch:\n{}",
        String::from_utf8_lossy(&actual),
        String::from_utf8_lossy(&expected)
    );
}

#[test]
fn compat_adversarial_prefix_chains() {
    if !vsearch_on_path() {
        eprintln!("SKIP: vsearch not on PATH");
        return;
    }
    // 20 groups × 4 seqs: lengths 80, 60, 45, 33 — guaranteed prefix chains.
    let bases = b"ACGT";
    let mut seed: u64 = 0xDEAD_BEEF_CAFE_1234;

    let tmp_in = tempfile::NamedTempFile::new().unwrap();
    let mut f = BufWriter::new(std::fs::File::create(tmp_in.path()).unwrap());

    for g in 0u32..20 {
        let base: Vec<u8> = (0..80)
            .map(|_| bases[(xorshift(&mut seed) % 4) as usize])
            .collect();
        for (suffix, len) in [("_80", 80usize), ("_60", 60), ("_45", 45), ("_33", 33)] {
            writeln!(f, ">g{g:02}{suffix}").unwrap();
            f.write_all(&base[..len]).unwrap();
            writeln!(f).unwrap();
        }
    }
    drop(f);

    let vs_out = tempfile::NamedTempFile::new().unwrap();
    let our_out = tempfile::NamedTempFile::new().unwrap();

    run_vsearch(tmp_in.path(), vs_out.path(), false);
    run_ours(tmp_in.path(), our_out.path(), false);

    let expected = std::fs::read(vs_out.path()).unwrap();
    let actual = std::fs::read(our_out.path()).unwrap();
    assert_eq!(
        actual,
        expected,
        "adversarial prefix-chains: output differs from vsearch\n\
         ours:\n{}\nvsearch:\n{}",
        String::from_utf8_lossy(&actual),
        String::from_utf8_lossy(&expected)
    );
}

#[test]
fn compat_heavy_ties_strcmp_order() {
    if !vsearch_on_path() {
        eprintln!("SKIP: vsearch not on PATH");
        return;
    }
    // 300 unique seqs with abundances 1-4, stressing strcmp tie-break ordering.
    let bases = b"ACGT";
    let mut seed: u64 = 0x4142_4344_4546_4748;

    let tmp_in = tempfile::NamedTempFile::new().unwrap();
    let mut f = BufWriter::new(std::fs::File::create(tmp_in.path()).unwrap());

    for i in 0usize..300 {
        let size = (i % 4) + 1;
        let seq: Vec<u8> = (0..60)
            .map(|_| bases[(xorshift(&mut seed) % 4) as usize])
            .collect();
        writeln!(f, ">s{};size={size}", i + 1).unwrap();
        f.write_all(&seq).unwrap();
        writeln!(f).unwrap();
    }
    drop(f);

    let vs_out = tempfile::NamedTempFile::new().unwrap();
    let our_out = tempfile::NamedTempFile::new().unwrap();

    run_vsearch(tmp_in.path(), vs_out.path(), true);
    run_ours(tmp_in.path(), our_out.path(), true);

    let expected = std::fs::read(vs_out.path()).unwrap();
    let actual = std::fs::read(our_out.path()).unwrap();
    assert_eq!(
        actual,
        expected,
        "heavy-ties (300 seqs, sizes 1-4, sizein): output differs from vsearch\n\
         ours:\n{}\nvsearch:\n{}",
        String::from_utf8_lossy(&actual),
        String::from_utf8_lossy(&expected)
    );
}

#[test]
fn compat_u_and_case_mixed() {
    if !vsearch_on_path() {
        eprintln!("SKIP: vsearch not on PATH");
        return;
    }
    // RNA-like long seqs with DNA lowercase prefix sequences.
    let tmp_in = tempfile::NamedTempFile::new().unwrap();
    let mut f = BufWriter::new(std::fs::File::create(tmp_in.path()).unwrap());

    let bases = b"ACG"; // no T — we put U in the long seq
    let mut seed: u64 = 0xFEED_FACE_DEAD_BEEF;
    for i in 0..10u32 {
        let base: Vec<u8> = (0..55)
            .map(|_| bases[(xorshift(&mut seed) % 3) as usize])
            .collect();
        // Long: replace random positions with U
        let long: Vec<u8> = base
            .iter()
            .map(|&b| if b == b'G' { b'U' } else { b })
            .collect();
        writeln!(f, ">rna_long_{i}").unwrap();
        f.write_all(&long).unwrap();
        writeln!(f).unwrap();

        // Prefix: use T instead of U, lowercase
        let pref: Vec<u8> = base[..40].iter().map(|&b| b.to_ascii_lowercase()).collect();
        writeln!(f, ">dna_pref_{i}").unwrap();
        f.write_all(&pref).unwrap();
        writeln!(f).unwrap();
    }
    drop(f);

    let vs_out = tempfile::NamedTempFile::new().unwrap();
    let our_out = tempfile::NamedTempFile::new().unwrap();

    run_vsearch(tmp_in.path(), vs_out.path(), false);
    run_ours(tmp_in.path(), our_out.path(), false);

    let expected = std::fs::read(vs_out.path()).unwrap();
    let actual = std::fs::read(our_out.path()).unwrap();
    assert_eq!(
        actual,
        expected,
        "U+case mixed: output differs from vsearch\n\
         ours:\n{}\nvsearch:\n{}",
        String::from_utf8_lossy(&actual),
        String::from_utf8_lossy(&expected)
    );
}

#[test]
fn compat_minseqlength_boundary() {
    if !vsearch_on_path() {
        eprintln!("SKIP: vsearch not on PATH");
        return;
    }
    // Seqs of lengths 31, 32, 33, 50.
    // Default minseqlength=32: 31-nt seq filtered, others kept.
    // 33-nt seq is prefix of 50-nt seq → merged.
    let base50 = "ACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTAC"; // 50
    let base33 = &base50[..33]; // prefix of base50
    let base31 = "ACGTACGTACGTACGTACGTACGTACGTACG"; // 31, filtered by minseqlength=32
    let independent32 = "GGGGCCCCGGGGCCCCGGGGCCCCGGGGCCCC"; // 32

    let tmp_in = tempfile::NamedTempFile::new().unwrap();
    let mut f = BufWriter::new(std::fs::File::create(tmp_in.path()).unwrap());
    writeln!(f, ">base50\n{base50}").unwrap();
    writeln!(f, ">base33\n{base33}").unwrap(); // prefix of base50 → merged
    writeln!(f, ">base31\n{base31}").unwrap(); // filtered
    writeln!(f, ">ind32\n{independent32}").unwrap(); // independent, survives
    drop(f);

    let vs_out = tempfile::NamedTempFile::new().unwrap();
    let our_out = tempfile::NamedTempFile::new().unwrap();

    run_vsearch(tmp_in.path(), vs_out.path(), false);
    run_ours(tmp_in.path(), our_out.path(), false);

    let expected = std::fs::read(vs_out.path()).unwrap();
    let actual = std::fs::read(our_out.path()).unwrap();
    assert_eq!(
        actual,
        expected,
        "minseqlength boundary: output differs from vsearch\n\
         ours:\n{}\nvsearch:\n{}",
        String::from_utf8_lossy(&actual),
        String::from_utf8_lossy(&expected)
    );
}
