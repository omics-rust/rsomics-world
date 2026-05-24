//! Compatibility test: run both rsomics-read-duplication and `RSeQC`
//! `read_duplication.py` on the golden fixture and assert BOTH output tables
//! are field-identical.
//!
//! Skipped if `read_duplication.py` is not found.

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
        let p = Path::new(dir).join("read_duplication.py");
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(out) = Command::new("which").arg("read_duplication.py").output()
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

fn read_xls(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
}

#[test]
fn both_tables_match_rseqc() {
    let Some(rseqc) = rseqc_bin() else {
        eprintln!("SKIP: read_duplication.py not found");
        return;
    };

    let bam = Path::new(GOLDEN).join("dup.bam");
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
        .expect("failed to run read_duplication.py");
    assert!(
        oracle_out.status.success(),
        "read_duplication.py failed: {}",
        String::from_utf8_lossy(&oracle_out.stderr)
    );

    // Run our binary.
    let our_bin = env!("CARGO_BIN_EXE_rsomics-read-duplication");
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
        .expect("failed to run rsomics-read-duplication");
    assert!(
        our_out.status.success(),
        "rsomics-read-duplication failed: {}",
        String::from_utf8_lossy(&our_out.stderr)
    );

    // Compare .seq.DupRate.xls field-by-field.
    let oracle_seq = read_xls(&tmp.path().join("oracle.seq.DupRate.xls"));
    let ours_seq = read_xls(&tmp.path().join("ours.seq.DupRate.xls"));
    assert_eq!(
        ours_seq, oracle_seq,
        "seq table mismatch:\n=== ours ===\n{ours_seq}\n=== rseqc ===\n{oracle_seq}"
    );

    // Compare .pos.DupRate.xls field-by-field.
    let oracle_pos = read_xls(&tmp.path().join("oracle.pos.DupRate.xls"));
    let ours_pos = read_xls(&tmp.path().join("ours.pos.DupRate.xls"));
    assert_eq!(
        ours_pos, oracle_pos,
        "pos table mismatch:\n=== ours ===\n{ours_pos}\n=== rseqc ===\n{oracle_pos}"
    );
}
