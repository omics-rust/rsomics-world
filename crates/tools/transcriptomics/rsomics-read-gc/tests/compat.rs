//! Compatibility test: run both `rsomics-read-gc` and `RSeQC` `read_GC.py`
//! on the golden fixture and assert the output tables are field-identical.
//!
//! Skipped if `read_GC.py` is not found.
//!
//! Comparison is field-level: both tables are parsed into sorted `(gc%, count)`
//! pairs and compared numerically.  `RSeQC` emits rows in Python dict insertion
//! order; we emit sorted; sorting both sides before comparison is the correct
//! approach for this dict-order difference.

use std::path::Path;
use std::process::Command;

const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");

fn rseqc_bin() -> Option<std::path::PathBuf> {
    let extra_dirs = [
        python_lib_bin(),
        "/usr/local/bin".to_string(),
        "/usr/bin".to_string(),
    ];
    for dir in &extra_dirs {
        let p = Path::new(dir).join("read_GC.py");
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(out) = Command::new("which").arg("read_GC.py").output()
        && out.status.success()
    {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !s.is_empty() {
            return Some(s.into());
        }
    }
    None
}

fn python_lib_bin() -> String {
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

/// Parse a `.GC.xls` file into a sorted `Vec<(f64, u64)>` — (GC%, count).
///
/// Skips the header line.  Sorts by GC% ascending so order-insensitive
/// comparison works regardless of `RSeQC` dict insertion order vs our sort order.
fn parse_xls(path: &Path) -> Vec<(f64, u64)> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    let mut rows: Vec<(f64, u64)> = text
        .lines()
        .skip(1) // skip header "GC%\tread_count"
        .filter(|l| !l.trim().is_empty())
        .map(|line| {
            let mut parts = line.splitn(2, '\t');
            let gc: f64 = parts
                .next()
                .unwrap_or("")
                .trim()
                .parse()
                .unwrap_or_else(|_| panic!("invalid GC% in line: {line:?}"));
            let count: u64 = parts
                .next()
                .unwrap_or("")
                .trim()
                .parse()
                .unwrap_or_else(|_| panic!("invalid count in line: {line:?}"));
            (gc, count)
        })
        .collect();
    rows.sort_unstable_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
    rows
}

#[test]
fn gc_xls_matches_rseqc() {
    let Some(rseqc) = rseqc_bin() else {
        eprintln!("SKIP: read_GC.py not found");
        return;
    };

    let bam = Path::new(GOLDEN).join("gc.bam");
    let tmp = tempfile::tempdir().unwrap();
    let oracle_prefix = tmp.path().join("oracle");
    let ours_prefix = tmp.path().join("ours");

    // Run RSeQC oracle.
    let oracle_out = Command::new(&rseqc)
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-o",
            oracle_prefix.to_str().unwrap(),
            "-q",
            "30",
        ])
        .output()
        .expect("failed to run read_GC.py");
    assert!(
        oracle_out.status.success(),
        "read_GC.py failed: {}",
        String::from_utf8_lossy(&oracle_out.stderr)
    );

    // Run our binary.
    let our_bin = env!("CARGO_BIN_EXE_rsomics-read-gc");
    let our_out = Command::new(our_bin)
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-o",
            ours_prefix.to_str().unwrap(),
            "--mapq",
            "30",
            "-t",
            "1",
        ])
        .output()
        .expect("failed to run rsomics-read-gc");
    assert!(
        our_out.status.success(),
        "rsomics-read-gc failed: {}",
        String::from_utf8_lossy(&our_out.stderr)
    );

    let oracle_xls = tmp.path().join("oracle.GC.xls");
    let ours_xls = tmp.path().join("ours.GC.xls");

    let oracle_rows = parse_xls(&oracle_xls);
    let ours_rows = parse_xls(&ours_xls);

    assert_eq!(
        ours_rows.len(),
        oracle_rows.len(),
        "row count mismatch: ours={} rseqc={}\nours: {ours_rows:?}\nrseqc: {oracle_rows:?}",
        ours_rows.len(),
        oracle_rows.len(),
    );

    // Full table assertion: every (GC%, count) pair must match.
    for (i, ((our_gc, our_cnt), (ref_gc, ref_cnt))) in
        ours_rows.iter().zip(oracle_rows.iter()).enumerate()
    {
        assert!(
            (our_gc - ref_gc).abs() < 1e-9,
            "row {i}: GC% mismatch: ours={our_gc:.2} rseqc={ref_gc:.2}"
        );
        assert_eq!(
            our_cnt, ref_cnt,
            "row {i}: count mismatch for GC%={our_gc:.2}: ours={our_cnt} rseqc={ref_cnt}"
        );
    }
}
