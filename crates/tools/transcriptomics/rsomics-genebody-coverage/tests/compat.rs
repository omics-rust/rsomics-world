//! Compatibility test: run both rsomics-genebody-coverage and `RSeQC`
//! `geneBody_coverage.py` on the golden fixture and assert the full
//! 100-value coverage profile is numerically identical.
//!
//! Skipped if `geneBody_coverage.py` is not on PATH.

use std::path::{Path, PathBuf};
use std::process::Command;

const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");

fn rseqc_bin() -> Option<PathBuf> {
    let extra_dirs = [
        dirs_search(),
        "/usr/local/bin".to_string(),
        "/usr/bin".to_string(),
    ];
    for dir in &extra_dirs {
        let p = Path::new(dir).join("geneBody_coverage.py");
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(out) = Command::new("which").arg("geneBody_coverage.py").output()
        && out.status.success()
    {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !s.is_empty() {
            return Some(s.into());
        }
    }
    None
}

fn dirs_search() -> String {
    if let Some(home) = std::env::var_os("HOME") {
        let base = Path::new(&home).join("Library").join("Python");
        if let Ok(rd) = std::fs::read_dir(&base) {
            let mut versions: Vec<String> = rd
                .flatten()
                .filter_map(|e| e.file_name().into_string().ok())
                .collect();
            versions.sort_unstable_by(|a, b| b.cmp(a));
            for v in versions {
                let dir = base.join(&v).join("bin");
                if dir.exists() {
                    return dir.to_string_lossy().into_owned();
                }
            }
        }
    }
    String::new()
}

/// Parse the coverage row (second row) of a `.geneBodyCoverage.txt` file.
/// Returns the 100 numeric values as f64.
fn parse_coverage_row(path: &Path) -> Vec<f64> {
    let content = std::fs::read_to_string(path).expect("reading coverage txt");
    let mut lines = content.lines();
    lines.next(); // skip header
    let row = lines.next().expect("expected coverage row");
    let fields: Vec<&str> = row.splitn(2, '\t').collect();
    assert_eq!(fields.len(), 2, "expected sample_name + values");
    fields[1]
        .split('\t')
        .map(|v| v.parse::<f64>().expect("parse coverage value"))
        .collect()
}

#[test]
fn coverage_profile_matches_rseqc() {
    let Some(rseqc) = rseqc_bin() else {
        eprintln!("SKIP: geneBody_coverage.py not found");
        return;
    };

    let bam = Path::new(GOLDEN).join("reads.bam");
    let bed = Path::new(GOLDEN).join("genes.bed12");

    let tmp = tempfile::tempdir().expect("tempdir");
    let oracle_prefix = tmp.path().join("oracle");

    // Run RSeQC oracle.
    let oracle_status = Command::new(&rseqc)
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-r",
            bed.to_str().unwrap(),
            "-o",
            oracle_prefix.to_str().unwrap(),
        ])
        .status()
        .expect("failed to run geneBody_coverage.py");
    assert!(
        oracle_status.success(),
        "geneBody_coverage.py exited non-zero"
    );

    let oracle_txt = oracle_prefix.with_file_name("oracle.geneBodyCoverage.txt");
    let oracle_vals = parse_coverage_row(&oracle_txt);
    assert_eq!(
        oracle_vals.len(),
        100,
        "oracle: expected 100 coverage values"
    );

    // Run our binary.
    let our_prefix = tmp.path().join("ours");
    let our_bin = env!("CARGO_BIN_EXE_rsomics-genebody-coverage");
    let our_status = Command::new(our_bin)
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-r",
            bed.to_str().unwrap(),
            "-o",
            our_prefix.to_str().unwrap(),
        ])
        .status()
        .expect("failed to run rsomics-genebody-coverage");
    assert!(
        our_status.success(),
        "rsomics-genebody-coverage exited non-zero"
    );

    let our_txt = our_prefix.with_file_name("ours.geneBodyCoverage.txt");
    let our_vals = parse_coverage_row(&our_txt);
    assert_eq!(our_vals.len(), 100, "ours: expected 100 coverage values");

    // Assert full 100-value profile matches numerically (integer values, tolerance=0).
    let mismatches: Vec<(usize, f64, f64)> = oracle_vals
        .iter()
        .zip(our_vals.iter())
        .enumerate()
        .filter(|(_, (o, u))| (*o - *u).abs() > 0.0)
        .map(|(i, (o, u))| (i + 1, *o, *u))
        .collect();

    assert!(
        mismatches.is_empty(),
        "coverage profile mismatch at {} position(s):\n{}\n\noracle:  {:?}\nours:    {:?}",
        mismatches.len(),
        mismatches
            .iter()
            .map(|(pos, o, u)| format!("  pct {pos}: oracle={o} ours={u}"))
            .collect::<Vec<_>>()
            .join("\n"),
        oracle_vals,
        our_vals,
    );
}
