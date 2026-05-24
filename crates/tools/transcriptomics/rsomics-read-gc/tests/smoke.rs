use std::num::NonZero;
use std::path::Path;

use rsomics_read_gc::compute_gc;

const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");

#[test]
fn golden_histogram_counts() {
    let bam = Path::new(GOLDEN).join("gc.bam");
    let hist = compute_gc(&bam, 30, NonZero::new(1).unwrap()).unwrap();

    // Verified against RSeQC read_GC.py 5.0.4 on the same fixture.
    // Filtered reads (mapq<30, unmapped, qcfail) are excluded.
    // N bases count in the denominator but not as GC.
    assert_eq!(hist.get("0.00").copied().unwrap_or(0), 2, "0.00% GC");
    assert_eq!(
        hist.get("18.18").copied().unwrap_or(0),
        2,
        "18.18% GC (2/11)"
    );
    assert_eq!(hist.get("33.33").copied().unwrap_or(0), 2, "33.33% GC");
    assert_eq!(hist.get("50.00").copied().unwrap_or(0), 5, "50.00% GC");
    assert_eq!(
        hist.get("50.98").copied().unwrap_or(0),
        1,
        "50.98% GC (102bp read)"
    );
    assert_eq!(hist.get("66.67").copied().unwrap_or(0), 1, "66.67% GC");
    assert_eq!(hist.get("100.00").copied().unwrap_or(0), 1, "100.00% GC");
    assert_eq!(hist.len(), 7, "total number of distinct GC% buckets");
}

#[test]
fn filtered_reads_excluded() {
    let bam = Path::new(GOLDEN).join("gc.bam");
    // With mapq threshold of 60 (above all read MAPQs), all reads are excluded.
    let hist = compute_gc(&bam, 60, NonZero::new(1).unwrap()).unwrap();
    assert!(hist.is_empty(), "mapq=60 should exclude all reads");
}

#[test]
fn n_bases_in_denominator() {
    let bam = Path::new(GOLDEN).join("gc.bam");
    let hist = compute_gc(&bam, 30, NonZero::new(1).unwrap()).unwrap();
    // 'GCNNNNNNNNN' (len=11, gc=2): 2/11*100 = 18.181818... → "18.18"
    // Two such reads → count=2
    assert_eq!(hist.get("18.18").copied().unwrap_or(0), 2);
    // 'NNNNNNNNNNN' (len=11, gc=0): 0/11*100 = 0.00
    assert_eq!(hist.get("0.00").copied().unwrap_or(0), 2);
}
