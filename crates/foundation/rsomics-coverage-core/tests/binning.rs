//! Exact bin-count assertions over a hand-built golden BAM. The five reads
//! (`tests/golden/small.sam`) place known coverage so the binning and the two
//! read totals are checked against arithmetic, not against a reference tool.

use std::num::NonZero;
use std::path::PathBuf;

use rsomics_coverage_core::{BinFilter, compute_coverage};

fn golden(n: &str) -> PathBuf {
    PathBuf::from(format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n))
}

fn one() -> NonZero<usize> {
    NonZero::new(1).unwrap()
}

// small.sam (1-based starts, all 50M):
//   chr1 (len 500): read1@1 → ref[0,50); read2@51, read3@51 → ref[50,100)
//   chr2 (len 300): read4@1 → ref[0,50);  read5@101 → ref[100,150)
#[test]
fn bin_counts_binsize50() {
    let cov = compute_coverage(&golden("small.bam"), 50, BinFilter::default(), one()).unwrap();

    assert_eq!(cov.chroms.len(), 2);
    assert_eq!(cov.total_binned, 5);
    assert_eq!(cov.total_mapped, 5);

    let chr1 = &cov.chroms[0];
    assert_eq!(chr1.name, "chr1");
    assert_eq!(chr1.chrom_len, 500);
    assert_eq!(chr1.bins.len(), 10);
    // bin 0 = read1; bin 1 = read2 + read3; rest zero.
    assert_eq!(chr1.bins[0], 1);
    assert_eq!(chr1.bins[1], 2);
    assert_eq!(chr1.bins[2..].iter().sum::<u32>(), 0);

    let chr2 = &cov.chroms[1];
    assert_eq!(chr2.name, "chr2");
    assert_eq!(chr2.chrom_len, 300);
    assert_eq!(chr2.bins.len(), 6);
    assert_eq!(chr2.bins[0], 1); // read4
    assert_eq!(chr2.bins[2], 1); // read5 @ ref 100
    assert_eq!(chr2.bins[1], 0);
}

#[test]
fn total_signal_matches_sum() {
    let cov = compute_coverage(&golden("small.bam"), 50, BinFilter::default(), one()).unwrap();
    // 1 + 2 (chr1) + 1 + 1 (chr2) = 5 bin-increments.
    assert_eq!(cov.total_signal(), 5);
}

#[test]
fn larger_bins_merge_coverage() {
    let cov = compute_coverage(&golden("small.bam"), 100, BinFilter::default(), one()).unwrap();
    let chr1 = &cov.chroms[0];
    assert_eq!(chr1.bins.len(), 5);
    // 100 bp bin 0 covers ref [0,100): read1 + read2 + read3 = 3.
    assert_eq!(chr1.bins[0], 3);
}

#[test]
fn min_mapq_filter_drops_reads() {
    // small.sam reads all have MAPQ 30; a threshold above that drops everything.
    let filter = BinFilter {
        skip_flags: 0,
        min_mapq: 60,
    };
    let cov = compute_coverage(&golden("small.bam"), 50, filter, one()).unwrap();
    assert_eq!(cov.total_binned, 0);
    assert_eq!(cov.total_mapped, 0);
    assert!(cov.chroms.iter().all(|c| c.bins.iter().all(|&b| b == 0)));
}
