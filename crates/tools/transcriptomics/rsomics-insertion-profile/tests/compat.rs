//! Compatibility test: run both `rsomics-insertion-profile` and RSeQC
//! `insertion_profile.py` on the golden fixture and assert the per-position
//! table is field-identical.
//!
//! Skipped if `insertion_profile.py` is not found.

use std::path::Path;
use std::process::Command;

const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");

fn oracle_bin() -> Option<std::path::PathBuf> {
    let extra_dirs = [
        python_lib_bin(),
        "/usr/local/bin".to_string(),
        "/usr/bin".to_string(),
    ];
    for dir in &extra_dirs {
        let p = Path::new(dir).join("insertion_profile.py");
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(out) = Command::new("which").arg("insertion_profile.py").output()
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

/// Parse a `.insertion_profile.xls` into `Vec<(usize, f64, f64)>`.
/// Skips the header line and section labels (`Read-1:`, `Read-2:`).
fn parse_xls(path: &Path) -> Vec<(usize, f64, f64)> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    text.lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .filter(|l| {
            let first = l.split('\t').next().unwrap_or("").trim();
            first.parse::<usize>().is_ok()
        })
        .map(|line| {
            let mut parts = line.splitn(3, '\t');
            let pos: usize = parts
                .next()
                .unwrap_or("")
                .trim()
                .parse()
                .unwrap_or_else(|_| panic!("invalid position in line: {line:?}"));
            let ins: f64 = parts
                .next()
                .unwrap_or("")
                .trim()
                .parse()
                .unwrap_or_else(|_| panic!("invalid Insert_nt in line: {line:?}"));
            let non_ins: f64 = parts
                .next()
                .unwrap_or("")
                .trim()
                .parse()
                .unwrap_or_else(|_| panic!("invalid Non_insert_nt in line: {line:?}"));
            (pos, ins, non_ins)
        })
        .collect()
}

#[test]
fn insertion_profile_xls_matches_oracle() {
    let Some(oracle) = oracle_bin() else {
        eprintln!("SKIP: insertion_profile.py not found");
        return;
    };

    let bam = Path::new(GOLDEN).join("ins.bam");
    let tmp = tempfile::tempdir().unwrap();
    let oracle_prefix = tmp.path().join("oracle");
    let ours_prefix = tmp.path().join("ours");

    let oracle_out = Command::new(&oracle)
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-s",
            "SE",
            "-o",
            oracle_prefix.to_str().unwrap(),
            "-q",
            "30",
        ])
        .output()
        .expect("failed to run insertion_profile.py");
    assert!(
        oracle_out.status.success(),
        "insertion_profile.py failed: {}",
        String::from_utf8_lossy(&oracle_out.stderr)
    );

    let our_bin = env!("CARGO_BIN_EXE_rsomics-insertion-profile");
    let our_out = Command::new(our_bin)
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-s",
            "SE",
            "-o",
            ours_prefix.to_str().unwrap(),
            "--mapq",
            "30",
            "-t",
            "1",
        ])
        .output()
        .expect("failed to run rsomics-insertion-profile");
    assert!(
        our_out.status.success(),
        "rsomics-insertion-profile failed: {}",
        String::from_utf8_lossy(&our_out.stderr)
    );

    let oracle_xls = tmp.path().join("oracle.insertion_profile.xls");
    let ours_xls = tmp.path().join("ours.insertion_profile.xls");

    let oracle_rows = parse_xls(&oracle_xls);
    let ours_rows = parse_xls(&ours_xls);

    assert_eq!(
        ours_rows.len(),
        oracle_rows.len(),
        "row count mismatch: ours={} oracle={}",
        ours_rows.len(),
        oracle_rows.len(),
    );

    for (i, ((our_pos, our_ins, our_ni), (ref_pos, ref_ins, ref_ni))) in
        ours_rows.iter().zip(oracle_rows.iter()).enumerate()
    {
        assert_eq!(our_pos, ref_pos, "row {i}: position mismatch");
        assert!(
            (our_ins - ref_ins).abs() < 1e-9,
            "row {i} pos={our_pos}: Insert_nt mismatch: ours={our_ins} oracle={ref_ins}"
        );
        assert!(
            (our_ni - ref_ni).abs() < 1e-9,
            "row {i} pos={our_pos}: Non_insert_nt mismatch: ours={our_ni} oracle={ref_ni}"
        );
    }
}
