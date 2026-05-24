use std::num::NonZero;
use std::path::Path;

use rsomics_bam_junctions::annotate_junctions;

const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");

#[test]
fn junction_counts_match_expected() {
    let bam = Path::new(GOLDEN).join("spliced.bam");
    let bed = Path::new(GOLDEN).join("genes.bed12");
    let counts = annotate_junctions(&bam, &bed, 50, 30, NonZero::new(1).unwrap()).unwrap();

    // 8 total N-ops (including the filtered one with length=30)
    assert_eq!(counts.total_events, 8, "total_events");
    assert_eq!(counts.filtered_events, 1, "filtered_events");

    // Events (per-read occurrences of passing introns)
    assert_eq!(counts.known_events, 4, "known_events");
    assert_eq!(counts.partial_novel_events, 2, "partial_novel_events");
    assert_eq!(counts.novel_events, 1, "novel_events");

    // Distinct junctions
    assert_eq!(counts.known_junctions, 3, "known_junctions");
    assert_eq!(counts.partial_novel_junctions, 2, "partial_novel_junctions");
    assert_eq!(counts.novel_junctions, 1, "novel_junctions");
    assert_eq!(counts.total_junctions(), 6, "total_junctions");
}

#[test]
fn stdout_format_matches_rseqc() {
    let bam = Path::new(GOLDEN).join("spliced.bam");
    let bed = Path::new(GOLDEN).join("genes.bed12");
    let counts = annotate_junctions(&bam, &bed, 50, 30, NonZero::new(1).unwrap()).unwrap();

    let mut stdout_buf = Vec::new();
    counts.write_rseqc_stdout(&mut stdout_buf).unwrap();
    let stdout_text = String::from_utf8(stdout_buf).unwrap();

    // Matches RSeQC: "total = 8\n" (total events including filtered)
    assert_eq!(
        stdout_text, "total = 8\n",
        "stdout mismatch: {stdout_text:?}"
    );
}

#[test]
fn stderr_format_contains_expected_lines() {
    let bam = Path::new(GOLDEN).join("spliced.bam");
    let bed = Path::new(GOLDEN).join("genes.bed12");
    let counts = annotate_junctions(&bam, &bed, 50, 30, NonZero::new(1).unwrap()).unwrap();

    let mut stderr_buf = Vec::new();
    counts.write_rseqc_stderr(&mut stderr_buf).unwrap();
    let text = String::from_utf8(stderr_buf).unwrap();

    assert!(
        text.contains("Total splicing  Events:\t8"),
        "events total: {text}"
    );
    assert!(
        text.contains("Known Splicing Events:\t4"),
        "known events: {text}"
    );
    assert!(
        text.contains("Partial Novel Splicing Events:\t2"),
        "partial events: {text}"
    );
    assert!(
        text.contains("Novel Splicing Events:\t1"),
        "novel events: {text}"
    );
    assert!(
        text.contains("Filtered Splicing Events:\t1"),
        "filtered: {text}"
    );
    assert!(
        text.contains("Total splicing  Junctions:\t6"),
        "junctions total: {text}"
    );
    assert!(
        text.contains("Known Splicing Junctions:\t3"),
        "known junctions: {text}"
    );
    assert!(
        text.contains("Partial Novel Splicing Junctions:\t2"),
        "partial junctions: {text}"
    );
    assert!(
        text.contains("Novel Splicing Junctions:\t1"),
        "novel junctions: {text}"
    );
}
