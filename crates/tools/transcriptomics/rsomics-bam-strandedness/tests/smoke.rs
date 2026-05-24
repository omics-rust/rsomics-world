use std::num::NonZero;
use std::path::Path;

use rsomics_bam_strandedness::{Protocol, infer_strandedness};

const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");

#[test]
fn fwd_pe_is_pairedend_forward() {
    let bam = Path::new(GOLDEN).join("fwd_pe.bam");
    let bed = Path::new(GOLDEN).join("genes.bed12");
    let result = infer_strandedness(&bam, &bed, 200_000, 30, NonZero::new(1).unwrap()).unwrap();

    assert_eq!(result.protocol, Protocol::PairEnd);
    assert_eq!(result.sampled, 20);
    // All 20 reads fall into spec1 (forward-stranded "1++,1--,2+-,2-+").
    assert!((result.spec1 - 1.0).abs() < 1e-9, "spec1={}", result.spec1);
    assert!(result.spec2.abs() < 1e-9, "spec2={}", result.spec2);
    assert!(result.other.abs() < 1e-9, "other={}", result.other);
}

#[test]
fn rseqc_output_format_matches_byte_for_byte() {
    let bam = Path::new(GOLDEN).join("fwd_pe.bam");
    let bed = Path::new(GOLDEN).join("genes.bed12");
    let result = infer_strandedness(&bam, &bed, 200_000, 30, NonZero::new(1).unwrap()).unwrap();

    let mut buf = Vec::new();
    result.write_rseqc(&mut buf).unwrap();
    let text = String::from_utf8(buf).unwrap();

    // RSeQC emits two bare newlines, then the header line, then the three fraction lines.
    assert!(
        text.starts_with("\n\nThis is PairEnd Data\n"),
        "got: {text:?}"
    );
    assert!(text.contains("Fraction of reads failed to determine: 0.0000"));
    assert!(text.contains("Fraction of reads explained by \"1++,1--,2+-,2-+\": 1.0000"));
    assert!(text.contains("Fraction of reads explained by \"1+-,1-+,2++,2--\": 0.0000"));
}
