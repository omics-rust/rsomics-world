use std::num::NonZero;
use std::path::Path;

use rsomics_clipping_profile::{Layout, compute_se_pub, write_xls};

const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");

fn golden_bam() -> std::path::PathBuf {
    Path::new(GOLDEN).join("clip.bam")
}

#[test]
fn se_profile_counts_match_expected() {
    let bam = golden_bam();
    let profile = compute_se_pub(&bam, 30, NonZero::new(1).unwrap()).unwrap();

    // 8 reads pass filter (2 skipped: low_mapq and unmapped).
    assert_eq!(profile.total_reads, 8);
    assert_eq!(profile.read_length, 100);
    assert_eq!(profile.clip_count.len(), 100);

    // Positions 0-4: fwd_5clip_10 + fwd_5clip_5 + fwd_both_5_5 = 3 clipped.
    for pos in 0..5 {
        assert_eq!(profile.clip_count[pos], 3, "pos {pos}");
    }
    // Positions 5-9: fwd_5clip_10 only = 1 clipped.
    for pos in 5..10 {
        assert_eq!(profile.clip_count[pos], 1, "pos {pos}");
    }
    // Positions 10-89: no clipping.
    for pos in 10..90 {
        assert_eq!(profile.clip_count[pos], 0, "pos {pos}");
    }
    // Positions 90-94: fwd_3clip_10 + rev_5clip_10 (mirrored) = 2 clipped.
    for pos in 90..95 {
        assert_eq!(profile.clip_count[pos], 2, "pos {pos}");
    }
    // Positions 95-99: fwd_3clip_10 + fwd_3clip_5 + fwd_both_5_5 + rev_5clip_10 = 4 clipped.
    for pos in 95..100 {
        assert_eq!(profile.clip_count[pos], 4, "pos {pos}");
    }
}

#[test]
fn write_xls_produces_correct_output() {
    let bam = golden_bam();
    let profile = compute_se_pub(&bam, 30, NonZero::new(1).unwrap()).unwrap();

    let tmp = tempfile::tempdir().unwrap();
    let prefix = tmp.path().join("out");
    write_xls(&prefix, Layout::Se, Some(&profile), None).unwrap();

    let xls = tmp.path().join("out.clipping_profile.xls");
    let text = std::fs::read_to_string(&xls).unwrap();
    let lines: Vec<&str> = text.lines().collect();

    assert_eq!(lines[0], "Position\tClipped_nt\tNon_clipped_nt");
    // Row for position 0: clip=3, non_clip=5 → "0\t3.0\t5.0"
    assert_eq!(lines[1], "0\t3.0\t5.0");
    // Row for position 10: clip=0, non_clip=8 → "10\t0\t8.0"
    assert_eq!(lines[11], "10\t0\t8.0");
    // 101 lines total: 1 header + 100 positions
    assert_eq!(lines.len(), 101);
}

#[test]
fn zero_fmt_is_bare_zero_nonzero_fmt_has_decimal() {
    // Verify formatting contract: 0 → "0", N>0 → "N.0"
    let bam = golden_bam();
    let profile = compute_se_pub(&bam, 30, NonZero::new(1).unwrap()).unwrap();

    let tmp = tempfile::tempdir().unwrap();
    let prefix = tmp.path().join("fmt");
    write_xls(&prefix, Layout::Se, Some(&profile), None).unwrap();

    let text = std::fs::read_to_string(tmp.path().join("fmt.clipping_profile.xls")).unwrap();
    // Position 10 has 0 clips: Clipped_nt must be bare "0", not "0.0"
    let row10 = text.lines().nth(11).unwrap();
    let cols: Vec<&str> = row10.splitn(3, '\t').collect();
    assert_eq!(cols[1], "0", "zero clip must be bare '0', not '0.0'");
    assert_eq!(cols[2], "8.0", "non-zero non-clip must be 'N.0' format");
}
