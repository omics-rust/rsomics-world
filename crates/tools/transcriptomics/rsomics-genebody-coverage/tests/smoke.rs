use std::path::Path;

use rsomics_genebody_coverage::{compute_coverage, load_transcripts};

const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");

#[test]
fn load_three_transcripts() {
    let bed = Path::new(GOLDEN).join("genes.bed12");
    let txs = load_transcripts(&bed, 100).unwrap();
    assert_eq!(txs.len(), 3, "expected 3 transcripts (all ≥ 100 nt mRNA)");
}

#[test]
fn coverage_has_100_positions() {
    let bed = Path::new(GOLDEN).join("genes.bed12");
    let bam = Path::new(GOLDEN).join("reads.bam");
    let txs = load_transcripts(&bed, 100).unwrap();
    let cov = compute_coverage(&bam, &txs).unwrap();
    assert_eq!(cov.len(), 100);
}

#[test]
fn gene_b_is_3prime_biased() {
    let bed = Path::new(GOLDEN).join("genes.bed12");
    let bam = Path::new(GOLDEN).join("reads.bam");
    let txs = load_transcripts(&bed, 100).unwrap();
    let cov = compute_coverage(&bam, &txs).unwrap();
    // GENE_B has heavy 3' bias: expect last 30 positions average > first 30 positions average.
    let first30: u64 = cov[..30].iter().sum();
    let last30: u64 = cov[70..].iter().sum();
    assert!(
        last30 > first30,
        "expected 3'-biased profile (last30={last30} > first30={first30})"
    );
}

#[test]
fn below_100nt_transcripts_filtered() {
    let bed = Path::new(GOLDEN).join("genes.bed12");
    // min_mrna_len=500 should exclude GENE_A (350bp) and GENE_C (400bp), keep only GENE_B (500bp).
    let txs = load_transcripts(&bed, 500).unwrap();
    assert_eq!(
        txs.len(),
        1,
        "only GENE_B (500bp) should pass the 500bp filter"
    );
}
