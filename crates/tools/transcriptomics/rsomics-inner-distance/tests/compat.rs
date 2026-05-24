//! Compatibility test: run both rsomics-inner-distance and `RSeQC` `inner_distance.py`
//! on the golden fixture and assert BOTH output files are field-identical.
//!
//! Skipped if `inner_distance.py` is not found in the search path.

use std::path::Path;
use std::process::Command;

const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");
const BIN: &str = env!("CARGO_BIN_EXE_rsomics-inner-distance");

fn rseqc_bin() -> Option<std::path::PathBuf> {
    if let Some(home) = std::env::var_os("HOME") {
        let base = Path::new(&home).join("Library").join("Python");
        if let Ok(rd) = std::fs::read_dir(&base) {
            let mut versions: Vec<String> = rd
                .flatten()
                .filter_map(|e| e.file_name().into_string().ok())
                .collect();
            versions.sort_unstable_by(|a, b| b.cmp(a));
            for v in versions {
                let p = base.join(&v).join("bin").join("inner_distance.py");
                if p.exists() {
                    return Some(p);
                }
            }
        }
    }
    for dir in ["/usr/local/bin", "/usr/bin"] {
        let p = Path::new(dir).join("inner_distance.py");
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(out) = Command::new("which").arg("inner_distance.py").output()
        && out.status.success()
    {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !s.is_empty() {
            return Some(s.into());
        }
    }
    None
}

fn read_file(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
}

/// Parse `inner_distance.txt` into sorted `(name, dist, category)` tuples.
fn parse_inner_distance(content: &str) -> Vec<(&str, &str, &str)> {
    let mut rows: Vec<(&str, &str, &str)> = content
        .lines()
        .filter_map(|l| {
            let parts: Vec<&str> = l.split('\t').collect();
            if parts.len() >= 3 {
                Some((parts[0], parts[1], parts[2]))
            } else {
                None
            }
        })
        .collect();
    rows.sort_unstable_by_key(|r| r.0);
    rows
}

/// Parse `inner_distance_freq.txt` into `(start, end, count)` tuples.
fn parse_freq(content: &str) -> Vec<(&str, &str, &str)> {
    content
        .lines()
        .filter_map(|l| {
            let parts: Vec<&str> = l.split('\t').collect();
            if parts.len() >= 3 {
                Some((parts[0], parts[1], parts[2]))
            } else {
                None
            }
        })
        .collect()
}

#[test]
#[allow(clippy::too_many_lines)]
fn both_outputs_match_rseqc() {
    let Some(rseqc) = rseqc_bin() else {
        eprintln!("SKIP: inner_distance.py not found");
        return;
    };

    let bam = Path::new(GOLDEN).join("pairs.bam");
    let bed = Path::new(GOLDEN).join("genes.bed12");
    if !bam.exists() || !bed.exists() {
        eprintln!("SKIP: golden fixture not found (run tests/make_golden.py first)");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let oracle_prefix = tmp.path().join("oracle").to_string_lossy().into_owned();
    let ours_prefix = tmp.path().join("ours").to_string_lossy().into_owned();

    let oracle_out = Command::new(&rseqc)
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-r",
            bed.to_str().unwrap(),
            "-o",
            &oracle_prefix,
            "-q",
            "30",
            "-k",
            "1000000",
            "-l",
            "-250",
            "-u",
            "250",
            "-s",
            "5",
        ])
        .output()
        .expect("failed to run inner_distance.py");
    assert!(
        oracle_out.status.success(),
        "inner_distance.py failed: {}",
        String::from_utf8_lossy(&oracle_out.stderr)
    );

    let our_out = Command::new(BIN)
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-r",
            bed.to_str().unwrap(),
            "-o",
            &ours_prefix,
            "--mapq",
            "30",
            "-k",
            "1000000",
            "-l",
            "-250",
            "-u",
            "250",
            "-s",
            "5",
        ])
        .output()
        .expect("failed to run rsomics-inner-distance");
    assert!(
        our_out.status.success(),
        "rsomics-inner-distance failed: {}",
        String::from_utf8_lossy(&our_out.stderr)
    );

    let oracle_txt = read_file(Path::new(&format!("{oracle_prefix}.inner_distance.txt")));
    let ours_txt = read_file(Path::new(&format!("{ours_prefix}.inner_distance.txt")));

    let oracle_pairs = parse_inner_distance(&oracle_txt);
    let ours_pairs = parse_inner_distance(&ours_txt);

    assert_eq!(
        oracle_pairs.len(),
        ours_pairs.len(),
        "inner_distance.txt: row count mismatch (oracle={}, ours={})",
        oracle_pairs.len(),
        ours_pairs.len()
    );

    for (i, (oracle_row, ours_row)) in oracle_pairs.iter().zip(ours_pairs.iter()).enumerate() {
        assert_eq!(oracle_row.0, ours_row.0, "row {i}: read name mismatch");
        assert_eq!(
            oracle_row.1, ours_row.1,
            "row {i} ({}): distance mismatch: oracle='{}' ours='{}'",
            oracle_row.0, oracle_row.1, ours_row.1
        );
        assert_eq!(
            oracle_row.2, ours_row.2,
            "row {i} ({}): category mismatch: oracle='{}' ours='{}'",
            oracle_row.0, oracle_row.2, ours_row.2
        );
    }

    eprintln!(
        "inner_distance.txt: {}/{} rows match",
        oracle_pairs.len(),
        oracle_pairs.len()
    );

    let oracle_freq = read_file(Path::new(&format!(
        "{oracle_prefix}.inner_distance_freq.txt"
    )));
    let ours_freq = read_file(Path::new(&format!("{ours_prefix}.inner_distance_freq.txt")));

    let oracle_hist = parse_freq(&oracle_freq);
    let ours_hist = parse_freq(&ours_freq);

    assert_eq!(
        oracle_hist.len(),
        ours_hist.len(),
        "inner_distance_freq.txt: bin count mismatch"
    );

    let mut mismatches = Vec::new();
    for (i, (oracle_bin, ours_bin)) in oracle_hist.iter().zip(ours_hist.iter()).enumerate() {
        if oracle_bin != ours_bin {
            mismatches.push(format!(
                "bin {i}: oracle='{}\t{}\t{}' ours='{}\t{}\t{}'",
                oracle_bin.0, oracle_bin.1, oracle_bin.2, ours_bin.0, ours_bin.1, ours_bin.2
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "inner_distance_freq.txt mismatches:\n{}",
        mismatches.join("\n")
    );

    eprintln!(
        "inner_distance_freq.txt: {}/{} bins match",
        oracle_hist.len(),
        oracle_hist.len()
    );
}

/// Assert the full frequency histogram against hardcoded values from `RSeQC` 5.0.4.
#[test]
fn freq_histogram_full_table() {
    let bam = Path::new(GOLDEN).join("pairs.bam");
    let bed = Path::new(GOLDEN).join("genes.bed12");
    if !bam.exists() || !bed.exists() {
        eprintln!("SKIP: golden fixture not found");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let prefix = tmp.path().join("hist_test").to_string_lossy().into_owned();

    let out = Command::new(BIN)
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-r",
            bed.to_str().unwrap(),
            "-o",
            &prefix,
            "--mapq",
            "30",
            "-l",
            "-250",
            "-u",
            "250",
            "-s",
            "5",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());

    let freq_path = format!("{prefix}.inner_distance_freq.txt");
    let freq = std::fs::read_to_string(&freq_path).unwrap();
    let hist: Vec<(&str, &str, &str)> = parse_freq(&freq);

    assert_eq!(hist.len(), 100, "expected 100 histogram bins");

    let nonzero: Vec<_> = hist.iter().filter(|b| b.2 != "0").collect();

    // Expected non-zero bins from `RSeQC` 5.0.4 on the golden fixture:
    //   -105  -100  1  (pair3 overlap d=-100)
    //    45    50   2  (pair1 mRNA=50, pair2 mRNA=50)
    //    95   100   1  (pair4 intergenic d=100)
    assert_eq!(
        nonzero.len(),
        3,
        "expected exactly 3 non-zero histogram bins, got {}: {:?}",
        nonzero.len(),
        nonzero
    );

    let find_bin =
        |start: &str| -> Option<(&str, &str, &str)> { hist.iter().find(|b| b.0 == start).copied() };

    let bin_m105 = find_bin("-105").expect("bin -105 not found");
    assert_eq!(
        bin_m105,
        ("-105", "-100", "1"),
        "bin -105..-100 count mismatch"
    );

    let bin_45 = find_bin("45").expect("bin 45 not found");
    assert_eq!(bin_45, ("45", "50", "2"), "bin 45..50 count mismatch");

    let bin_95 = find_bin("95").expect("bin 95 not found");
    assert_eq!(bin_95, ("95", "100", "1"), "bin 95..100 count mismatch");
}
