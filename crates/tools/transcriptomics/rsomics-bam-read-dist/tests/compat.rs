//! Compatibility test: run both rsomics-bam-read-dist and `RSeQC` `read_distribution.py`
//! on the golden fixture and assert the outputs are byte-identical.
//!
//! Skipped if `RSeQC` is not on PATH.

use std::path::Path;
use std::process::Command;

const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");

fn rseqc_bin() -> Option<std::path::PathBuf> {
    let extra_dirs = [
        dirs_search(),
        "/usr/local/bin".to_string(),
        "/usr/bin".to_string(),
    ];
    for dir in &extra_dirs {
        let p = Path::new(dir).join("read_distribution.py");
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(out) = Command::new("which").arg("read_distribution.py").output()
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

#[test]
fn output_matches_rseqc() {
    let Some(rseqc) = rseqc_bin() else {
        eprintln!("SKIP: read_distribution.py not found");
        return;
    };

    let bam = Path::new(GOLDEN).join("reads.bam");
    let bed = Path::new(GOLDEN).join("genes.bed12");

    let oracle_out = Command::new(&rseqc)
        .args(["-i", bam.to_str().unwrap(), "-r", bed.to_str().unwrap()])
        .output()
        .expect("failed to run read_distribution.py");
    assert!(
        oracle_out.status.success(),
        "read_distribution.py failed: {}",
        String::from_utf8_lossy(&oracle_out.stderr)
    );
    let oracle_stdout = String::from_utf8(oracle_out.stdout).unwrap();

    let our_bin = env!("CARGO_BIN_EXE_rsomics-bam-read-dist");
    let our_out = Command::new(our_bin)
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-r",
            bed.to_str().unwrap(),
            "-t",
            "1",
        ])
        .output()
        .expect("failed to run rsomics-bam-read-dist");
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
