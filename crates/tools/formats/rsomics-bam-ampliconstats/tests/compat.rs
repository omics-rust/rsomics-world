use std::path::PathBuf;
use std::process::{Command, Stdio};

fn ours() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_rsomics-bam-ampliconstats"))
}

fn samtools_version() -> Option<String> {
    let out = Command::new("samtools")
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    // First line: "samtools X.Y.Z"
    let ver = text.lines().next()?;
    Some(ver.to_string())
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name)
}

fn run_ours(primers: &std::path::Path, bam: &std::path::Path) -> String {
    let out = Command::new(ours())
        .arg(primers)
        .arg(bam)
        .output()
        .expect("spawn rsomics-bam-ampliconstats");
    assert!(
        out.status.success(),
        "rsomics-bam-ampliconstats failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn run_samtools(primers: &std::path::Path, bam: &std::path::Path) -> String {
    let out = Command::new("samtools")
        .args([
            "ampliconstats",
            primers.to_str().unwrap(),
            bam.to_str().unwrap(),
        ])
        .output()
        .expect("spawn samtools ampliconstats");
    assert!(
        out.status.success(),
        "samtools ampliconstats failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// Skip lines that inherently differ between implementations
/// (SS\tSamtools version and SS\tCommand line).
fn compare_ignoring_metadata(ours: &str, theirs: &str) {
    fn filter_lines(s: &str) -> Vec<&str> {
        s.lines()
            .filter(|l| {
                !l.starts_with("SS\tSamtools version") && !l.starts_with("SS\tCommand line")
            })
            .collect()
    }

    let ours_lines = filter_lines(ours);
    let theirs_lines = filter_lines(theirs);

    assert_eq!(
        ours_lines,
        theirs_lines,
        "output differs (excluding metadata lines):\n--- ours ({} lines) ---\n{}\n--- samtools ({} lines) ---\n{}",
        ours_lines.len(),
        ours_lines.join("\n"),
        theirs_lines.len(),
        theirs_lines.join("\n"),
    );
}

#[test]
fn ampliconstats_matches_samtools() {
    let ver = match samtools_version() {
        Some(v) => v,
        None => {
            eprintln!("samtools not on PATH — skipping compat test");
            return;
        }
    };

    let primers = fixture("primers.bed");
    let bam = fixture("amplicons.bam");
    if !primers.exists() || !bam.exists() {
        eprintln!("golden fixtures missing — skipping compat test");
        return;
    }

    // Version gate: requires samtools 1.13+ for multi-ref output format.
    // Extract major.minor from "samtools X.Y.Z".
    let ver_num: Vec<u32> = ver
        .split_whitespace()
        .nth(1)
        .unwrap_or("0.0.0")
        .split('.')
        .filter_map(|p| p.parse().ok())
        .collect();
    let (major, minor) = (
        ver_num.first().copied().unwrap_or(0),
        ver_num.get(1).copied().unwrap_or(0),
    );
    if major < 1 || (major == 1 && minor < 13) {
        eprintln!("samtools {ver} < 1.13 — multi-ref format not supported, skipping");
        return;
    }

    let ours = run_ours(&primers, &bam);
    let theirs = run_samtools(&primers, &bam);

    compare_ignoring_metadata(&ours, &theirs);
}
