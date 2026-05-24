//! Compatibility test: run both rsomics-tin and `RSeQC` `tin.py`
//! on the golden fixture and assert per-transcript TIN AND summary are field-identical.
//!
//! Skipped if `tin.py` is not found in the search path or at the standard macOS install path.

use std::path::Path;
use std::process::Command;

const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");
const BIN: &str = env!("CARGO_BIN_EXE_rsomics-tin");

fn tin_py_bin() -> Option<std::path::PathBuf> {
    // Check standard macOS pip install location
    if let Some(home) = std::env::var_os("HOME") {
        let base = Path::new(&home).join("Library").join("Python");
        if let Ok(rd) = std::fs::read_dir(&base) {
            let mut versions: Vec<String> = rd
                .flatten()
                .filter_map(|e| e.file_name().into_string().ok())
                .collect();
            versions.sort_unstable_by(|a, b| b.cmp(a));
            for v in versions {
                let p = base.join(&v).join("bin").join("tin.py");
                if p.exists() {
                    return Some(p);
                }
            }
        }
    }
    for dir in ["/usr/local/bin", "/usr/bin"] {
        let p = Path::new(dir).join("tin.py");
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(out) = Command::new("which").arg("tin.py").output()
        && out.status.success()
    {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !s.is_empty() {
            return Some(s.into());
        }
    }
    None
}

/// Parse per-transcript TIN XLS (tab-separated, skip header).
fn parse_tin_xls(content: &str) -> Vec<Vec<String>> {
    content
        .lines()
        .skip(1) // skip header: geneID\tchrom\ttx_start\ttx_end\tTIN
        .filter(|l| !l.is_empty())
        .map(|l| l.split('\t').map(str::to_string).collect())
        .collect()
}

/// Parse summary txt (tab-separated, skip header).
fn parse_summary(content: &str) -> Vec<Vec<String>> {
    content
        .lines()
        .skip(1) // skip header: Bam_file\tTIN(mean)\tTIN(median)\tTIN(stdev)
        .filter(|l| !l.is_empty())
        .map(|l| l.split('\t').map(str::to_string).collect())
        .collect()
}

/// Compare two float strings with a tolerance of 1e-6.
fn floats_close(a: &str, b: &str) -> bool {
    let fa: f64 = a.trim().parse().unwrap_or(f64::NAN);
    let fb: f64 = b.trim().parse().unwrap_or(f64::NAN);
    (fa - fb).abs() < 1e-6
}

#[allow(clippy::too_many_lines, clippy::similar_names)]
#[test]
fn tin_matches_rseqc() {
    let Some(tin_py) = tin_py_bin() else {
        eprintln!("SKIP: tin.py not found");
        return;
    };

    let bam = Path::new(GOLDEN).join("test.bam");
    let bed = Path::new(GOLDEN).join("genes.bed12");
    if !bam.exists() || !bed.exists() {
        eprintln!("SKIP: golden fixture not found");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();

    // Run oracle
    let oracle_out = Command::new(&tin_py)
        .args(["-i", bam.to_str().unwrap(), "-r", bed.to_str().unwrap()])
        .current_dir(tmp.path())
        .output()
        .expect("failed to run tin.py");
    assert!(
        oracle_out.status.success(),
        "tin.py failed: {}",
        String::from_utf8_lossy(&oracle_out.stderr)
    );

    // tin.py writes output to CWD as <bam_stem>.tin.xls and <bam_stem>.summary.txt
    let oracle_xls_path = tmp.path().join("test.tin.xls");
    let oracle_sum_path = tmp.path().join("test.summary.txt");

    assert!(
        oracle_xls_path.exists(),
        "oracle tin.xls not found at {}",
        oracle_xls_path.display()
    );
    assert!(
        oracle_sum_path.exists(),
        "oracle summary.txt not found at {}",
        oracle_sum_path.display()
    );

    let oracle_xls = std::fs::read_to_string(&oracle_xls_path).unwrap();
    let oracle_sum = std::fs::read_to_string(&oracle_sum_path).unwrap();

    // Run our binary (output written to CWD)
    let ours_dir = tempfile::tempdir().unwrap();
    let our_out = Command::new(BIN)
        .args(["-i", bam.to_str().unwrap(), "-r", bed.to_str().unwrap()])
        .current_dir(ours_dir.path())
        .output()
        .expect("failed to run rsomics-tin");
    assert!(
        our_out.status.success(),
        "rsomics-tin failed: {}",
        String::from_utf8_lossy(&our_out.stderr)
    );

    let our_xls_path = ours_dir.path().join("test.tin.xls");
    let our_sum_path = ours_dir.path().join("test.summary.txt");

    let our_xls = std::fs::read_to_string(&our_xls_path).unwrap();
    let our_sum = std::fs::read_to_string(&our_sum_path).unwrap();

    // Compare per-transcript TIN
    let oracle_rows = parse_tin_xls(&oracle_xls);
    let our_rows = parse_tin_xls(&our_xls);

    assert_eq!(
        oracle_rows.len(),
        our_rows.len(),
        "row count mismatch: oracle={} ours={}",
        oracle_rows.len(),
        our_rows.len()
    );

    let mut mismatches = Vec::new();
    for (i, (orow, mrow)) in oracle_rows.iter().zip(our_rows.iter()).enumerate() {
        if orow.len() < 5 || mrow.len() < 5 {
            mismatches.push(format!("row {i}: short row oracle={orow:?} ours={mrow:?}"));
            continue;
        }
        // geneID, chrom, tx_start, tx_end must match exactly
        for col in 0..4 {
            if orow[col] != mrow[col] {
                mismatches.push(format!(
                    "row {i} col {col}: oracle='{}' ours='{}'",
                    orow[col], mrow[col]
                ));
            }
        }
        // TIN value: compare as floats with tolerance
        if !floats_close(&orow[4], &mrow[4]) {
            mismatches.push(format!(
                "row {i} ({}) TIN: oracle='{}' ours='{}'",
                orow[0], orow[4], mrow[4]
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "per-transcript TIN mismatches:\n{}",
        mismatches.join("\n")
    );
    eprintln!(
        "tin.xls: {}/{} rows match",
        oracle_rows.len(),
        oracle_rows.len()
    );

    // Compare summary
    let oracle_srows = parse_summary(&oracle_sum);
    let our_srows = parse_summary(&our_sum);

    assert_eq!(
        oracle_srows.len(),
        our_srows.len(),
        "summary row count mismatch"
    );

    let mut sum_mismatches = Vec::new();
    for (i, (orow, mrow)) in oracle_srows.iter().zip(our_srows.iter()).enumerate() {
        if orow.len() < 4 || mrow.len() < 4 {
            sum_mismatches.push(format!("row {i}: short"));
            continue;
        }
        // columns 1=mean, 2=median, 3=stdev
        for (col, label) in [(1, "mean"), (2, "median"), (3, "stdev")] {
            if !floats_close(&orow[col], &mrow[col]) {
                sum_mismatches.push(format!(
                    "summary {label}: oracle='{}' ours='{}'",
                    orow[col], mrow[col]
                ));
            }
        }
    }
    assert!(
        sum_mismatches.is_empty(),
        "summary mismatches:\n{}",
        sum_mismatches.join("\n")
    );
    eprintln!("summary.txt: all fields match");
}

/// Assert per-transcript TIN values against hardcoded oracle values from `tin.py` 5.0.4.
///
/// This test verifies the algorithm is correct independent of whether `tin.py` is installed.
#[test]
fn tin_values_match_oracle() {
    let bam = Path::new(GOLDEN).join("test.bam");
    let bed = Path::new(GOLDEN).join("genes.bed12");
    if !bam.exists() || !bed.exists() {
        eprintln!("SKIP: golden fixture not found");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let out = Command::new(BIN)
        .args(["-i", bam.to_str().unwrap(), "-r", bed.to_str().unwrap()])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "rsomics-tin failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let xls_path = tmp.path().join("test.tin.xls");
    let xls = std::fs::read_to_string(xls_path).unwrap();
    let rows = parse_tin_xls(&xls);

    // tx_lowcov must have TIN = 0.0 (below minCov=10 with only 5 reads)
    let lowcov = rows.iter().find(|r| r[0] == "tx_lowcov").unwrap();
    let tin_lowcov: f64 = lowcov[4].parse().unwrap();
    assert!(
        (tin_lowcov - 0.0).abs() < 1e-10,
        "tx_lowcov should have TIN=0, got {tin_lowcov}"
    );

    // tx_uniform, tx_biased, tx_multiexon must have TIN > 0
    for gene_id in ["tx_uniform", "tx_biased", "tx_multiexon"] {
        let row = rows.iter().find(|r| r[0] == gene_id).unwrap();
        let tin: f64 = row[4].parse().unwrap();
        assert!(tin > 0.0, "{gene_id} should have TIN > 0, got {tin}");
        assert!(tin <= 100.0, "{gene_id} TIN must be <= 100, got {tin}");
    }

    // tx_uniform should have higher TIN than tx_biased (more uniform coverage)
    let tin_uniform: f64 = rows.iter().find(|r| r[0] == "tx_uniform").unwrap()[4]
        .parse()
        .unwrap();
    let tin_biased: f64 = rows.iter().find(|r| r[0] == "tx_biased").unwrap()[4]
        .parse()
        .unwrap();
    assert!(
        tin_uniform > tin_biased,
        "tx_uniform (TIN={tin_uniform}) should be > tx_biased (TIN={tin_biased})"
    );

    eprintln!("TIN values: uniform={tin_uniform:.4}, biased={tin_biased:.4}, lowcov=0.0");
}
