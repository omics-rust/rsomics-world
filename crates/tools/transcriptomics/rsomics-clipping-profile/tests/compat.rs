//! Compatibility test: run both `rsomics-clipping-profile` and `RSeQC`
//! `clipping_profile.py` on the golden fixture and assert the per-position
//! table is field-identical.
//!
//! Skipped if `clipping_profile.py` is not found on PATH or in the
//! Python user-install bin directory.

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
        let p = Path::new(dir).join("clipping_profile.py");
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(out) = Command::new("which").arg("clipping_profile.py").output()
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

/// Parse a `.clipping_profile.xls` file into a `Vec<(usize, f64, f64)>`
/// (`position`, `clipped_nt`, `non_clipped_nt`) rows.  Skips the header line and
/// any section-label lines (e.g. "Read-1:").
fn parse_xls(path: &Path) -> Vec<(usize, f64, f64)> {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    text.lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .filter(|l| {
            // Skip PE section headers like "Read-1:" or "Read-2:".
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
            let clipped: f64 = parts
                .next()
                .unwrap_or("")
                .trim()
                .parse()
                .unwrap_or_else(|_| panic!("invalid Clipped_nt in line: {line:?}"));
            let non_clipped: f64 = parts
                .next()
                .unwrap_or("")
                .trim()
                .parse()
                .unwrap_or_else(|_| panic!("invalid Non_clipped_nt in line: {line:?}"));
            (pos, clipped, non_clipped)
        })
        .collect()
}

#[test]
fn clipping_profile_xls_matches_oracle() {
    let Some(oracle) = oracle_bin() else {
        eprintln!("SKIP: clipping_profile.py not found");
        return;
    };

    let bam = Path::new(GOLDEN).join("clip.bam");
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
        .expect("failed to run clipping_profile.py");
    assert!(
        oracle_out.status.success(),
        "clipping_profile.py failed: {}",
        String::from_utf8_lossy(&oracle_out.stderr)
    );

    let our_bin = env!("CARGO_BIN_EXE_rsomics-clipping-profile");
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
        .expect("failed to run rsomics-clipping-profile");
    assert!(
        our_out.status.success(),
        "rsomics-clipping-profile failed: {}",
        String::from_utf8_lossy(&our_out.stderr)
    );

    let oracle_xls = tmp.path().join("oracle.clipping_profile.xls");
    let ours_xls = tmp.path().join("ours.clipping_profile.xls");

    let oracle_rows = parse_xls(&oracle_xls);
    let ours_rows = parse_xls(&ours_xls);

    assert_eq!(
        ours_rows.len(),
        oracle_rows.len(),
        "row count mismatch: ours={} oracle={}",
        ours_rows.len(),
        oracle_rows.len(),
    );

    for (i, ((our_pos, our_clip, our_nc), (ref_pos, ref_clip, ref_nc))) in
        ours_rows.iter().zip(oracle_rows.iter()).enumerate()
    {
        assert_eq!(our_pos, ref_pos, "row {i}: position mismatch");
        assert!(
            (our_clip - ref_clip).abs() < 1e-9,
            "row {i} pos={our_pos}: Clipped_nt mismatch: ours={our_clip} oracle={ref_clip}"
        );
        assert!(
            (our_nc - ref_nc).abs() < 1e-9,
            "row {i} pos={our_pos}: Non_clipped_nt mismatch: ours={our_nc} oracle={ref_nc}"
        );
    }
}
