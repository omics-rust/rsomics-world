use std::io::BufReader;
use std::process::Command;

fn sort_binary() -> std::path::PathBuf {
    let exe = env!("CARGO_BIN_EXE_rsomics-fastx-sort");
    exe.into()
}

fn golden(name: &str) -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/golden");
    p.push(name);
    p
}

#[test]
fn smoke_sortbysize_basic_order() {
    let input = golden("basic_size.fasta");
    let out = tempfile::NamedTempFile::new().unwrap();
    let status = Command::new(sort_binary())
        .args([
            input.to_str().unwrap(),
            "-o",
            out.path().to_str().unwrap(),
            "--mode",
            "size",
            "--sizein",
            "--sizeout",
            "-q",
        ])
        .status()
        .unwrap();
    assert!(status.success(), "binary exited non-zero");
    let content = std::fs::read_to_string(out.path()).unwrap();
    // seq_a and seq_b both size=5; seq_a < seq_b lexicographically → seq_a first
    let pos_a = content.find(">seq_a").unwrap();
    let pos_b = content.find(">seq_b").unwrap();
    let pos_c = content.find(">seq_c").unwrap();
    assert!(pos_a < pos_b, "seq_a should precede seq_b (lex tie-break)");
    assert!(pos_b < pos_c, "size-5 records should precede size-3");
}

#[test]
fn smoke_sortbylength_basic_order() {
    let input = golden("basic_len.fasta");
    let out = tempfile::NamedTempFile::new().unwrap();
    let status = Command::new(sort_binary())
        .args([
            input.to_str().unwrap(),
            "-o",
            out.path().to_str().unwrap(),
            "--mode",
            "length",
            "-q",
        ])
        .status()
        .unwrap();
    assert!(status.success(), "binary exited non-zero");
    let content = std::fs::read_to_string(out.path()).unwrap();
    let pos_long = content.find(">long_seq").unwrap();
    let pos_medium = content.find(">medium_seq").unwrap();
    let pos_short = content.find(">short_seq").unwrap();
    assert!(pos_long < pos_medium, "long_seq should precede medium_seq");
    assert!(
        pos_medium < pos_short,
        "medium_seq should precede short_seq"
    );
}

#[test]
fn smoke_sizeout_strips_and_reappends() {
    use rsomics_fastx_sort::{read_records, sort_by_size};

    let input = ">seq1;other=x;size=3;extra=y\nACGTACGTACGTACGTACGTACGTACGTACGTACGT\n\
                 >seq2;size=5\nTGCATGCATGCATGCATGCATGCATGCATGCATGCA\n";
    let mut reader = BufReader::new(input.as_bytes());
    let (mut records, _) = read_records(&mut reader, 1, 50000).unwrap();
    sort_by_size(&mut records);
    // seq2 (size=5) should be first
    assert_eq!(records[0].abundance, 5);
    // stripped_header should have size= removed
    assert_eq!(records[0].stripped_header, "seq2");
    // seq1 stripped: "seq1;other=x;extra=y"
    assert_eq!(records[1].stripped_header, "seq1;other=x;extra=y");
    // raw_header preserves original (with ;size=)
    assert_eq!(records[1].raw_header, "seq1;other=x;size=3;extra=y");
}

#[test]
fn smoke_sortbysize_tie_break_raw_header() {
    use rsomics_fastx_sort::{read_records, sort_by_size};

    // Both size=5. Tie-break on raw header.
    // "c;a_extra=1;size=5" vs "c;size=5":
    // raw compare: "c;a_extra=1;size=5" < "c;size=5" (';a' < ';s') → first entry comes first
    let input = ">c;size=5\nACGTACGTACGTACGTACGTACGTACGTACGTACGT\n\
                 >c;a_extra=1;size=5\nTGCATGCATGCATGCATGCATGCATGCATGCATGCA\n";
    let mut reader = BufReader::new(input.as_bytes());
    let (mut records, _) = read_records(&mut reader, 1, 50000).unwrap();
    sort_by_size(&mut records);
    // "c;a_extra=1;size=5" has 'a' after ';' while "c;size=5" has 's' → a < s → first
    assert_eq!(
        records[0].raw_header, "c;a_extra=1;size=5",
        "raw header tie-break: 'c;a_extra' < 'c;size' → first"
    );
}

#[test]
fn smoke_sortbylength_three_tier_tiebreak() {
    use rsomics_fastx_sort::{read_records, sort_by_length};

    // Same length, different sizes
    let seq40 = "A".repeat(40);
    let input = format!(">b_seq;size=2\n{seq40}\n>a_seq;size=2\n{seq40}\n>c_seq;size=3\n{seq40}\n");
    let mut reader = BufReader::new(input.as_bytes());
    let (mut records, _) = read_records(&mut reader, 1, 50000).unwrap();
    sort_by_length(&mut records);
    // All same length → size desc → c_seq(3) first
    assert_eq!(records[0].raw_header, "c_seq;size=3");
    // Among size=2: a_seq < b_seq lexicographically
    assert_eq!(records[1].raw_header, "a_seq;size=2");
    assert_eq!(records[2].raw_header, "b_seq;size=2");
}

#[test]
fn smoke_minseqlength_filter() {
    use rsomics_fastx_sort::read_records;

    // short (8 nt) + adequate (40 nt)
    let input = ">short\nACGTACGT\n>adequate\n".to_string() + &"A".repeat(40) + "\n";
    let mut reader = BufReader::new(input.as_bytes());
    let (records, discarded) = read_records(&mut reader, 32, 50000).unwrap();
    assert_eq!(discarded, 1, "short sequence should be discarded");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].raw_header, "adequate");
}

#[test]
fn smoke_maxseqlength_filter() {
    use rsomics_fastx_sort::read_records;

    let long_seq = "A".repeat(50001);
    let input = format!(">toolong\n{long_seq}\n>ok\n{}\n", "A".repeat(40));
    let mut reader = BufReader::new(input.as_bytes());
    let (records, discarded) = read_records(&mut reader, 1, 50000).unwrap();
    assert_eq!(discarded, 1, "overlong sequence should be discarded");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].raw_header, "ok");
}

#[test]
fn smoke_no_sizein_defaults_to_one() {
    use rsomics_fastx_sort::{read_records, sort_by_size};

    // Sequence without ;size= annotation gets abundance 1
    let input = ">no_annotation\n".to_string() + &"A".repeat(40) + "\n";
    let mut reader = BufReader::new(input.as_bytes());
    let (mut records, _) = read_records(&mut reader, 1, 50000).unwrap();
    sort_by_size(&mut records);
    assert_eq!(records[0].abundance, 1);
}

#[test]
fn smoke_case_preserved_in_output() {
    use rsomics_fastx_sort::read_records;

    let input = ">seq\nacgtACGTacgtACGTacgtACGTacgtACGTacgtACGT\n";
    let mut reader = BufReader::new(input.as_bytes());
    let (records, _) = read_records(&mut reader, 1, 50000).unwrap();
    // Sequence bytes are preserved as-is (no normalisation for sort output)
    assert_eq!(records[0].seq, b"acgtACGTacgtACGTacgtACGTacgtACGTacgtACGT");
}

#[test]
fn smoke_cli_debug_assert() {
    let status = Command::new(sort_binary()).arg("--help").status().unwrap();
    let _ = status;
}
