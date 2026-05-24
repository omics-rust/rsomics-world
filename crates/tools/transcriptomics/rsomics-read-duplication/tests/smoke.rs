use std::num::NonZero;
use std::path::Path;

use rsomics_read_duplication::compute_duplication;

const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");

#[test]
fn golden_seq_histogram() {
    let bam = Path::new(GOLDEN).join("dup.bam");
    let hists = compute_duplication(&bam, 30, NonZero::new(1).unwrap()).unwrap();

    // 3 copies of SEQ_A → seq_hist[3] = 1
    // 2 copies of SEQ_B → seq_hist[2] = 1
    // 1 copy of SEQ_C and 1 copy of SEQ_D → seq_hist[1] = 2
    // Low-MAPQ and QC-fail reads are excluded.
    assert_eq!(hists.seq.get(&1).copied().unwrap_or(0), 2, "seq occ=1");
    assert_eq!(hists.seq.get(&2).copied().unwrap_or(0), 1, "seq occ=2");
    assert_eq!(hists.seq.get(&3).copied().unwrap_or(0), 1, "seq occ=3");
    assert_eq!(hists.seq.len(), 3, "seq histogram row count");
}

#[test]
fn golden_pos_histogram() {
    let bam = Path::new(GOLDEN).join("dup.bam");
    let hists = compute_duplication(&bam, 30, NonZero::new(1).unwrap()).unwrap();

    // 3 reads at chr1:1000 with 24M → pos_hist[3] = 1
    // 2 reads at chr1:2000 with 24M → pos_hist[2] = 1
    // 1 read at chr1:3000, 1 at chr1:4000  → pos_hist[1] = 2
    assert_eq!(hists.pos.get(&1).copied().unwrap_or(0), 2, "pos occ=1");
    assert_eq!(hists.pos.get(&2).copied().unwrap_or(0), 1, "pos occ=2");
    assert_eq!(hists.pos.get(&3).copied().unwrap_or(0), 1, "pos occ=3");
    assert_eq!(hists.pos.len(), 3, "pos histogram row count");
}
