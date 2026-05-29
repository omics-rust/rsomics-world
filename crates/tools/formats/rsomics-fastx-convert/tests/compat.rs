use std::path::PathBuf;

use rsomics_fastx_convert::{fa2fq, fq2fa, fx2tab, phred_recode, tab2fx};

fn golden(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name)
}

fn read_golden(name: &str) -> Vec<u8> {
    std::fs::read(golden(name)).unwrap()
}

// ---- fq2fa ----

#[test]
fn fq2fa_strips_quality() {
    let fq = read_golden("test.fq");
    let mut out = Vec::new();
    fq2fa(fq.as_slice(), &mut out).unwrap();
    let s = String::from_utf8(out).unwrap();
    assert!(s.starts_with(">seq1\n"), "expected FASTA, got: {s:?}");
    assert!(!s.contains('+'), "quality separator should be absent");
    assert!(s.contains("ACGTACGT"));
}

#[test]
fn fq2fa_passes_fasta_through() {
    let fa = read_golden("test.fa");
    let mut out = Vec::new();
    fq2fa(fa.as_slice(), &mut out).unwrap();
    let s = String::from_utf8(out).unwrap();
    assert!(s.starts_with(">seq1\n"));
    assert!(s.contains("ACGTACGT"));
}

// ---- fa2fq ----

#[test]
fn fa2fq_adds_dummy_quality() {
    let fa = read_golden("test.fa");
    let mut out = Vec::new();
    fa2fq(fa.as_slice(), &mut out).unwrap();
    let s = String::from_utf8(out).unwrap();
    assert!(s.starts_with("@seq1\n"), "expected FASTQ, got: {s:?}");
    assert!(s.contains('+'), "quality separator must be present");
    // dummy quality = 'I' repeated to match seq length
    assert!(s.contains("IIIIIIII"));
}

#[test]
fn fa2fq_preserves_existing_quality() {
    let fq = read_golden("test.fq");
    let mut out = Vec::new();
    fa2fq(fq.as_slice(), &mut out).unwrap();
    let s = String::from_utf8(out).unwrap();
    // seq2 has quality 'H', must be preserved
    assert!(s.contains("HHHHHHHH"));
}

// ---- fx2tab ----

#[test]
fn fx2tab_fasta_two_columns() {
    let fa = read_golden("test.fa");
    let mut out = Vec::new();
    fx2tab(fa.as_slice(), &mut out).unwrap();
    let s = String::from_utf8(out).unwrap();
    let first = s.lines().next().unwrap();
    let cols: Vec<&str> = first.split('\t').collect();
    assert_eq!(cols.len(), 2, "FASTA tab output should have 2 columns");
    assert_eq!(cols[0], "seq1");
    assert_eq!(cols[1], "ACGTACGT");
}

#[test]
fn fx2tab_fastq_three_columns() {
    let fq = read_golden("test.fq");
    let mut out = Vec::new();
    fx2tab(fq.as_slice(), &mut out).unwrap();
    let s = String::from_utf8(out).unwrap();
    let first = s.lines().next().unwrap();
    let cols: Vec<&str> = first.split('\t').collect();
    assert_eq!(cols.len(), 3, "FASTQ tab output should have 3 columns");
    assert_eq!(cols[0], "seq1");
    assert_eq!(cols[1], "ACGTACGT");
    assert_eq!(cols[2], "IIIIIIII");
}

// ---- tab2fx ----

#[test]
fn tab2fx_two_cols_produces_fasta() {
    let tab = b"seq1\tACGTACGT\nseq2\tTTTTGGGG\n";
    let mut out = Vec::new();
    tab2fx(tab.as_ref(), &mut out).unwrap();
    let s = String::from_utf8(out).unwrap();
    assert!(s.starts_with(">seq1\n"));
    assert!(s.contains("ACGTACGT"));
}

#[test]
fn tab2fx_three_cols_produces_fastq() {
    let tab = b"seq1\tACGTACGT\tIIIIIIII\n";
    let mut out = Vec::new();
    tab2fx(tab.as_ref(), &mut out).unwrap();
    let s = String::from_utf8(out).unwrap();
    assert!(s.starts_with("@seq1\n"));
    assert!(s.contains("IIIIIIII"));
}

// ---- phred recode ----

#[test]
fn phred_33_to_64() {
    // 'I' = ASCII 73 = Phred+33 Q40 → Phred+64 Q40 = 'h' = ASCII 104
    let fq = b"@r\nACGT\n+\nIIII\n";
    let mut out = Vec::new();
    phred_recode(fq.as_ref(), &mut out, 33).unwrap();
    let s = String::from_utf8(out).unwrap();
    // quality should now be 'hhhh' (73 - 33 + 64 = 104 = 'h')
    assert!(s.contains("hhhh"), "expected 'hhhh' in {s:?}");
}

#[test]
fn phred_64_to_33() {
    // 'h' = ASCII 104 = Phred+64 Q40 → Phred+33 Q40 = 'I' = ASCII 73
    let fq = b"@r\nACGT\n+\nhhhh\n";
    let mut out = Vec::new();
    phred_recode(fq.as_ref(), &mut out, 64).unwrap();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("IIII"), "expected 'IIII' in {s:?}");
}

#[test]
fn phred_rejects_fasta() {
    let fa = b">seq\nACGT\n";
    let mut out = Vec::new();
    let res = phred_recode(fa.as_ref(), &mut out, 33);
    assert!(res.is_err(), "should fail on FASTA input");
}

// ---- roundtrip ----

#[test]
fn fq2fa_then_fa2fq_roundtrip_seq() {
    let fq = read_golden("test.fq");
    let mut fa_buf = Vec::new();
    fq2fa(fq.as_slice(), &mut fa_buf).unwrap();
    let mut fq2_buf = Vec::new();
    fa2fq(fa_buf.as_slice(), &mut fq2_buf).unwrap();
    let orig = String::from_utf8(fq).unwrap();
    let roundtrip = String::from_utf8(fq2_buf).unwrap();
    // Sequences must match; quality will differ (dummy I vs original)
    let orig_seqs: Vec<&str> = orig
        .lines()
        .enumerate()
        .filter(|(i, _)| i % 4 == 1)
        .map(|(_, l)| l)
        .collect();
    let rt_seqs: Vec<&str> = roundtrip
        .lines()
        .enumerate()
        .filter(|(i, _)| i % 4 == 1)
        .map(|(_, l)| l)
        .collect();
    assert_eq!(
        orig_seqs, rt_seqs,
        "sequences must survive FASTQ→FASTA→FASTQ"
    );
}

#[test]
fn fx2tab_then_tab2fx_roundtrip_fastq() {
    let fq = read_golden("test.fq");
    let mut tab = Vec::new();
    fx2tab(fq.as_slice(), &mut tab).unwrap();
    let mut fq2 = Vec::new();
    tab2fx(tab.as_slice(), &mut fq2).unwrap();
    let s = String::from_utf8(fq2).unwrap();
    assert!(s.starts_with("@seq1\n"));
    assert!(s.contains("ACGTACGT"));
    assert!(s.contains("IIIIIIII"));
}

// ---- CLI: exit non-zero on bad subcommand ----

#[test]
fn exit_nonzero_on_bad_input() {
    use std::process::Command;
    let bin = env!("CARGO_BIN_EXE_rsomics-fastx-convert");
    // Feed invalid input to fq2fa
    let status = Command::new(bin)
        .args(["fq2fa", "--input", "/nonexistent/path.fq"])
        .status()
        .expect("spawn binary");
    assert!(!status.success());
}
