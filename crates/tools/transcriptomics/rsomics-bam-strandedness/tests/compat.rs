//! Compatibility test: run both rsomics-bam-strandedness and `RSeQC` `infer_experiment.py`
//! on the golden fixture and assert the outputs are byte-identical.
//!
//! Skipped if `RSeQC` is not on PATH.

use std::path::Path;
use std::process::Command;

const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");

fn rseqc_bin() -> Option<std::path::PathBuf> {
    // Common install locations: ~/Library/Python/<v>/bin, ~/.local/bin, /usr/local/bin.
    let extra_dirs = [
        dirs_search(),
        "/usr/local/bin".to_string(),
        "/usr/bin".to_string(),
    ];
    for dir in &extra_dirs {
        let p = Path::new(dir).join("infer_experiment.py");
        if p.exists() {
            return Some(p);
        }
    }
    // Fall back to PATH search.
    if let Ok(out) = Command::new("which").arg("infer_experiment.py").output()
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
    // ~/Library/Python/<major.minor>/bin on macOS
    if let Some(home) = std::env::var_os("HOME") {
        let base = Path::new(&home).join("Library").join("Python");
        if let Ok(rd) = std::fs::read_dir(&base) {
            let mut versions: Vec<String> = rd
                .flatten()
                .filter_map(|e| e.file_name().into_string().ok())
                .collect();
            versions.sort_unstable_by(|a, b| b.cmp(a)); // descending: newest first
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

#[test]
fn output_matches_rseqc() {
    let Some(rseqc) = rseqc_bin() else {
        eprintln!("SKIP: infer_experiment.py not found");
        return;
    };

    let bam = Path::new(GOLDEN).join("fwd_pe.bam");
    let bed = Path::new(GOLDEN).join("genes.bed12");

    // Run RSeQC oracle.
    let oracle_out = Command::new(&rseqc)
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-r",
            bed.to_str().unwrap(),
            "-s",
            "200000",
        ])
        .output()
        .expect("failed to run infer_experiment.py");
    assert!(
        oracle_out.status.success(),
        "infer_experiment.py failed: {}",
        String::from_utf8_lossy(&oracle_out.stderr)
    );
    let oracle_stdout = String::from_utf8(oracle_out.stdout).unwrap();

    // Run our binary.
    let our_bin = env!("CARGO_BIN_EXE_rsomics-bam-strandedness");
    let our_out = Command::new(our_bin)
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-r",
            bed.to_str().unwrap(),
            "-s",
            "200000",
            "-t",
            "1",
        ])
        .output()
        .expect("failed to run rsomics-bam-strandedness");
    assert!(
        our_out.status.success(),
        "binary failed: {}",
        String::from_utf8_lossy(&our_out.stderr)
    );
    let our_stdout = String::from_utf8(our_out.stdout).unwrap();

    assert_eq!(
        our_stdout, oracle_stdout,
        "output mismatch:\n=== ours ===\n{our_stdout}\n=== rseqc ===\n{oracle_stdout}"
    );
}
