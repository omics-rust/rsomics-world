//! Compatibility test: run both rsomics-bam-junctions and `RSeQC` `junction_annotation.py`
//! on the golden fixture and assert stdout counts match.
//!
//! Skipped if `junction_annotation.py` is not on PATH or in the common install dirs.

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
        let p = Path::new(dir).join("junction_annotation.py");
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(out) = Command::new("which").arg("junction_annotation.py").output()
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
fn stdout_matches_rseqc() {
    let Some(rseqc) = rseqc_bin() else {
        eprintln!("SKIP: junction_annotation.py not found");
        return;
    };

    let bam = Path::new(GOLDEN).join("spliced.bam");
    let bed = Path::new(GOLDEN).join("genes.bed12");

    // RSeQC writes the junction XLS to the working directory; use a temp dir.
    let tmp = tempfile::tempdir().expect("tempdir");
    let prefix = tmp.path().join("junc_out");

    let oracle = Command::new(&rseqc)
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-r",
            bed.to_str().unwrap(),
            "-o",
            prefix.to_str().unwrap(),
            "-m",
            "50",
            "-q",
            "30",
        ])
        .output()
        .expect("failed to run junction_annotation.py");
    assert!(
        oracle.status.success(),
        "junction_annotation.py failed: {}",
        String::from_utf8_lossy(&oracle.stderr)
    );
    // RSeQC stdout contains "total = N\n" (possibly preceded by Rscript device output).
    let oracle_stdout = String::from_utf8(oracle.stdout).unwrap();
    // Extract the "total = N" line from RSeQC stdout (Rscript may emit extra lines).
    let oracle_total_line = oracle_stdout
        .lines()
        .find(|l| l.starts_with("total = "))
        .expect("RSeQC stdout missing 'total = ' line")
        .to_string();

    // Run our binary.
    let our_bin = env!("CARGO_BIN_EXE_rsomics-bam-junctions");
    let ours = Command::new(our_bin)
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-r",
            bed.to_str().unwrap(),
            "-m",
            "50",
            "--mapq",
            "30",
            "-t",
            "1",
        ])
        .output()
        .expect("failed to run rsomics-bam-junctions");
    assert!(
        ours.status.success(),
        "rsomics-bam-junctions failed: {}",
        String::from_utf8_lossy(&ours.stderr)
    );
    let our_stdout = String::from_utf8(ours.stdout).unwrap();
    let our_total_line = our_stdout
        .lines()
        .find(|l| l.starts_with("total = "))
        .expect("our stdout missing 'total = ' line")
        .to_string();

    assert_eq!(
        our_total_line, oracle_total_line,
        "stdout 'total' mismatch:\n=== ours ===\n{our_total_line}\n=== rseqc ===\n{oracle_total_line}"
    );

    // The meaningful output — the known/partial-novel/novel/filtered classification
    // of events and junctions — goes to stderr on both tools; assert it byte-identical.
    let classification = |stderr: &[u8]| -> Vec<String> {
        String::from_utf8_lossy(stderr)
            .lines()
            .filter(|l| l.contains("Splicing Events:") || l.contains("Splicing Junctions:"))
            .map(|l| l.trim().to_string())
            .collect()
    };
    let our_class = classification(&ours.stderr);
    let oracle_class = classification(&oracle.stderr);
    assert!(
        !our_class.is_empty(),
        "no classification lines in our stderr:\n{}",
        String::from_utf8_lossy(&ours.stderr)
    );
    assert_eq!(
        our_class, oracle_class,
        "junction classification mismatch:\n=== ours ===\n{our_class:#?}\n=== rseqc ===\n{oracle_class:#?}"
    );
}
