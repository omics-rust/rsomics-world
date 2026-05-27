/// Compatibility test against RSeQC RPKM_saturation.py.
///
/// RSeQC's subsampling is non-deterministic (Python `random` without a seed).
/// Byte-exact comparison is therefore impossible. Instead we verify:
///   1. Both tools exit successfully on the same input.
///   2. Gene rows appear in the same order with matching BED6 prefix columns.
///   3. RPKM values at the 100% fraction are within 1% relative tolerance
///      (the 100% fraction is deterministic — it uses all reads).
///   4. Raw counts at 100% match exactly (counts are deterministic at 100%).
///
/// At sub-100% fractions we only check that the values are positive and
/// finite (structural sanity), not their exact magnitude, because the two
/// tools use different RNGs.
use std::path::{Path, PathBuf};
use std::process::Command;

fn ours() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_rsomics-rpkm-saturation"))
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name)
}

fn rseqc_on_path() -> bool {
    Command::new("RPKM_saturation.py")
        .arg("--version")
        .output()
        .is_ok()
}

fn run_ours(bam: &Path, bed: &Path, prefix: &str) {
    let out = Command::new(ours())
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-r",
            bed.to_str().unwrap(),
            "-o",
            prefix,
            "--mapq",
            "0",
            "--seed",
            "42",
        ])
        .output()
        .expect("spawn rsomics-rpkm-saturation");
    assert!(
        out.status.success(),
        "rsomics-rpkm-saturation failed (exit {:?}):\nstdout: {}\nstderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}

fn run_rseqc(bam: &Path, bed: &Path, prefix: &str) {
    let out = Command::new("RPKM_saturation.py")
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-r",
            bed.to_str().unwrap(),
            "-o",
            prefix,
            "-q",
            "0",
        ])
        .output()
        .expect("spawn RPKM_saturation.py");
    assert!(
        out.status.success(),
        "RPKM_saturation.py failed (exit {:?}):\nstdout: {}\nstderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}

fn parse_xls(path: &str) -> Vec<(String, Vec<f64>)> {
    let content = std::fs::read_to_string(path).expect("read xls");
    content
        .lines()
        .filter(|l| !l.starts_with('#'))
        .map(|l| {
            let cols: Vec<&str> = l.split('\t').collect();
            assert!(cols.len() >= 7, "too few columns in {path}: {l}");
            // BED6 key: chr\tstart\tend\tname\tscore\tstrand
            let key = cols[..6].join("\t");
            let vals: Vec<f64> = cols[6..]
                .iter()
                .map(|s| s.trim().parse::<f64>().unwrap_or(f64::NAN))
                .collect();
            (key, vals)
        })
        .collect()
}

fn last_col_index_of(path: &str) -> usize {
    let content = std::fs::read_to_string(path).expect("read xls");
    let header = content.lines().next().unwrap_or("");
    // Count fraction columns (skip first 6 BED columns in header)
    header.split('\t').count().saturating_sub(7)
}

#[test]
fn compat_vs_rseqc() {
    let bam = fixture("small.bam");
    let bed = fixture("small.bed12");
    if !bam.exists() || !bed.exists() {
        eprintln!("golden fixtures missing — skipping compat test");
        return;
    }
    if !rseqc_on_path() {
        eprintln!("RPKM_saturation.py not on PATH — skipping compat test");
        return;
    }

    let tmp = tempfile::tempdir().expect("tempdir");
    let ours_prefix = tmp.path().join("ours").to_str().unwrap().to_string();
    let rseqc_prefix = tmp.path().join("rseqc").to_str().unwrap().to_string();

    run_ours(&bam, &bed, &ours_prefix);
    run_rseqc(&bam, &bed, &rseqc_prefix);

    let ours_rpkm = format!("{ours_prefix}.eRPKM.xls");
    let rseqc_rpkm = format!("{rseqc_prefix}.eRPKM.xls");
    let ours_raw = format!("{ours_prefix}.rawCount.xls");
    let rseqc_raw = format!("{rseqc_prefix}.rawCount.xls");

    let ours_rpkm_rows = parse_xls(&ours_rpkm);
    let rseqc_rpkm_rows = parse_xls(&rseqc_rpkm);
    let ours_raw_rows = parse_xls(&ours_raw);
    let rseqc_raw_rows = parse_xls(&rseqc_raw);

    assert_eq!(
        ours_rpkm_rows.len(),
        rseqc_rpkm_rows.len(),
        "gene count mismatch in eRPKM"
    );
    assert_eq!(
        ours_raw_rows.len(),
        rseqc_raw_rows.len(),
        "gene count mismatch in rawCount"
    );

    let last_frac_col = last_col_index_of(&ours_rpkm);

    for (i, ((ours_key, ours_vals), (rseqc_key, rseqc_vals))) in ours_rpkm_rows
        .iter()
        .zip(rseqc_rpkm_rows.iter())
        .enumerate()
    {
        assert_eq!(ours_key, rseqc_key, "gene key mismatch at row {i}");

        // Check all RPKM values are non-negative and finite
        for (fi, &v) in ours_vals.iter().enumerate() {
            assert!(
                v.is_finite() && v >= 0.0,
                "ours RPKM[{i}][{fi}] = {v} is invalid"
            );
        }

        // At 100% fraction (last column): within 1% relative tolerance.
        // Both tools process all reads at 100%, so values should agree closely
        // modulo floating-point ordering differences in the RPKM formula.
        let ours_100 = ours_vals.get(last_frac_col).copied().unwrap_or(0.0);
        let rseqc_100 = rseqc_vals.get(last_frac_col).copied().unwrap_or(0.0);
        if rseqc_100 > 0.0 {
            let rel_diff = (ours_100 - rseqc_100).abs() / rseqc_100;
            assert!(
                rel_diff < 0.01,
                "RPKM@100% mismatch at row {i} ({ours_key}): ours={ours_100:.4}, rseqc={rseqc_100:.4}, rel_diff={rel_diff:.4}"
            );
        }
    }

    // Raw counts at 100% must match exactly.
    for (i, ((ours_key, ours_vals), (_rseqc_key, rseqc_vals))) in
        ours_raw_rows.iter().zip(rseqc_raw_rows.iter()).enumerate()
    {
        let ours_100 = ours_vals.get(last_frac_col).copied().unwrap_or(0.0);
        let rseqc_100 = rseqc_vals.get(last_frac_col).copied().unwrap_or(0.0);
        assert_eq!(
            ours_100 as u64, rseqc_100 as u64,
            "rawCount@100% mismatch at row {i} ({ours_key}): ours={ours_100}, rseqc={rseqc_100}"
        );
    }

    eprintln!(
        "compat OK: {} genes, {} fraction columns checked",
        ours_rpkm_rows.len(),
        last_frac_col + 1
    );
}
