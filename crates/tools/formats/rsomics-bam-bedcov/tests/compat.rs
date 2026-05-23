use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-bedcov"))
}

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn samtools_version() -> Option<String> {
    let out = Command::new("samtools").arg("--version").output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    Some(text.lines().next().unwrap_or("").to_string())
}

fn samtools_available() -> bool {
    Command::new("samtools")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Build the golden fixture (sorted BAM + index + BED) if not already present.
/// This requires samtools at fixture-build time and is idempotent.
fn ensure_golden(dir: &Path) -> (PathBuf, PathBuf) {
    let bam = dir.join("small.bam");
    let bed = dir.join("regions.bed");
    if bam.exists() && bed.exists() && dir.join("small.bam.bai").exists() {
        return (bam, bed);
    }

    // A minimal SAM with a few reads across two regions.
    // chr1 is 1000 bp long; chr2 is 500 bp.
    let sam = "\
@HD\tVN:1.6\tSO:coordinate\n\
@SQ\tSN:chr1\tLN:1000\n\
@SQ\tSN:chr2\tLN:500\n\
r1\t0\tchr1\t100\t60\t48M\t*\t0\t0\tAATTCCGGAATTCCGGAATTCCGGAATTCCGGAATTCCGGAATTCCGG\t*\n\
r2\t0\tchr1\t130\t60\t30M\t*\t0\t0\tAATTCCGGAATTCCGGAATTCCGGAATTCC\t*\n\
r3\t0\tchr1\t200\t60\t40M\t*\t0\t0\tAATTCCGGAATTCCGGAATTCCGGAATTCCGGAATTCCGG\t*\n\
r4\t0\tchr2\t50\t60\t60M\t*\t0\t0\tAATTCCGGAATTCCGGAATTCCGGAATTCCGGAATTCCGGAATTCCGGAATTCCGGAATT\t*\n\
r5\t4\tchr1\t300\t0\t48M\t*\t0\t0\tAATTCCGGAATTCCGGAATTCCGGAATTCCGGAATTCCGGAATTCCGG\t*\n\
";

    let sam_path = dir.join("small.sam");
    std::fs::write(&sam_path, sam).expect("write SAM");

    let sort_status = Command::new("samtools")
        .args(["sort", "-o"])
        .arg(&bam)
        .arg(&sam_path)
        .status()
        .expect("samtools sort");
    assert!(sort_status.success(), "samtools sort failed");

    let index_status = Command::new("samtools")
        .arg("index")
        .arg(&bam)
        .status()
        .expect("samtools index");
    assert!(index_status.success(), "samtools index failed");

    // BED regions: one spanning r1+r2 overlap area, one spanning r3, one on chr2.
    // Using 0-based half-open coords matching BED spec.
    let bed_content = "chr1\t100\t180\tregion_A\t0\t.\nchr1\t190\t250\tregion_B\t0\t.\nchr2\t40\t120\tregion_C\t0\t.\n";
    std::fs::write(&bed, bed_content).expect("write BED");

    (bam, bed)
}

/// The coverage column(s) from samtools bedcov output (tab-separated, one per BAM).
/// We compare only these appended columns, not any potential whitespace differences
/// in the original BED columns (samtools passes the BED through verbatim).
fn extract_coverage_columns(output: &str, bed_cols: usize) -> Vec<Vec<u64>> {
    output
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .map(|l| {
            let fields: Vec<&str> = l.split('\t').collect();
            fields[bed_cols..]
                .iter()
                .map(|v| v.trim().parse::<u64>().unwrap())
                .collect()
        })
        .collect()
}

#[test]
fn matches_samtools_bedcov() {
    if !samtools_available() {
        eprintln!("skipping: samtools not found");
        return;
    }
    let version = samtools_version().unwrap_or_default();
    eprintln!("samtools version: {version}");

    let dir = golden_dir();
    let (bam, bed) = ensure_golden(&dir);

    let ours = bin()
        .arg(&bed)
        .arg(&bam)
        .output()
        .expect("run rsomics-bam-bedcov");
    assert!(
        ours.status.success(),
        "rsomics-bam-bedcov failed: {}",
        String::from_utf8_lossy(&ours.stderr)
    );

    let theirs = Command::new("samtools")
        .arg("bedcov")
        .arg(&bed)
        .arg(&bam)
        .output()
        .expect("run samtools bedcov");
    assert!(
        theirs.status.success(),
        "samtools bedcov failed: {}",
        String::from_utf8_lossy(&theirs.stderr)
    );

    let ours_str = String::from_utf8_lossy(&ours.stdout);
    let theirs_str = String::from_utf8_lossy(&theirs.stdout);

    // The BED has 6 columns; the 7th is the coverage column added by bedcov.
    let ours_cov = extract_coverage_columns(&ours_str, 6);
    let theirs_cov = extract_coverage_columns(&theirs_str, 6);

    assert_eq!(
        ours_cov.len(),
        theirs_cov.len(),
        "region count mismatch: ours={} theirs={}",
        ours_cov.len(),
        theirs_cov.len()
    );

    for (i, (ours_row, theirs_row)) in ours_cov.iter().zip(theirs_cov.iter()).enumerate() {
        assert_eq!(
            ours_row, theirs_row,
            "coverage mismatch at region {i}: ours={ours_row:?} theirs={theirs_row:?}\n\
             ours output:\n{ours_str}\nsamtools output:\n{theirs_str}"
        );
    }

    eprintln!("compat OK: {ours_str}");
}

#[test]
fn output_has_correct_column_count() {
    if !samtools_available() {
        eprintln!("skipping: samtools not found");
        return;
    }
    let dir = golden_dir();
    let (bam, bed) = ensure_golden(&dir);

    let out = bin()
        .arg(&bed)
        .arg(&bam)
        .output()
        .expect("run rsomics-bam-bedcov");
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);

    // Each output line should have 7 columns (6 BED + 1 coverage).
    for line in text.lines() {
        let cols = line.split('\t').count();
        assert_eq!(cols, 7, "expected 7 columns, got {cols}: {line:?}");
    }
}
