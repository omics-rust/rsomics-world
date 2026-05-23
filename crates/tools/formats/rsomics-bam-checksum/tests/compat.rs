use std::path::PathBuf;
use std::process::Command;

fn ours() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_rsomics-bam-checksum"))
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name)
}

fn samtools_version() -> Option<String> {
    let out = Command::new("samtools").arg("--version").output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let first = text.lines().next()?;
    // "samtools 1.23.1"
    Some(first.trim().to_owned())
}

fn run_ours(input: &std::path::Path) -> String {
    let out = Command::new(ours())
        .arg(input)
        .output()
        .expect("spawn rsomics-bam-checksum");
    assert!(
        out.status.success(),
        "rsomics-bam-checksum failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn run_samtools(input: &std::path::Path) -> String {
    let out = Command::new("samtools")
        .args(["checksum", input.to_str().unwrap()])
        .output()
        .expect("spawn samtools checksum");
    assert!(
        out.status.success(),
        "samtools checksum failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// Extract only the data lines (skip the file-path header line which differs).
fn data_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|l| !l.starts_with("# Checksum 1.0 for file:"))
        .collect()
}

#[test]
fn checksum_matches_samtools_golden() {
    let ver = match samtools_version() {
        Some(v) => v,
        None => {
            eprintln!("samtools not on PATH — skipping compat test");
            return;
        }
    };
    // Version-gate: this test is validated against samtools 1.23.x.
    if !ver.contains("1.23") {
        eprintln!("samtools version is {ver}; test validated against 1.23.x — skipping");
        return;
    }

    let input = fixture("small.bam");
    if !input.exists() {
        eprintln!("golden fixture small.bam missing — skipping compat test");
        return;
    }

    let ours = run_ours(&input);
    let theirs = run_samtools(&input);

    assert_eq!(
        data_lines(&ours),
        data_lines(&theirs),
        "output differs (samtools {ver}):\n--- ours ---\n{ours}\n--- samtools ---\n{theirs}"
    );
}

/// Order-independence: checksum of coord-sorted == checksum of name-sorted BAM.
#[test]
fn checksum_is_order_independent() {
    let ver = match samtools_version() {
        Some(v) => v,
        None => {
            eprintln!("samtools not on PATH — skipping order-independence test");
            return;
        }
    };
    if !ver.contains("1.23") {
        eprintln!("samtools version {ver} not validated — skipping");
        return;
    }

    let input = fixture("small.bam");
    if !input.exists() {
        eprintln!("golden fixture missing — skipping order-independence test");
        return;
    }

    // Sort by name into a temp file.
    let tmp = tempfile::NamedTempFile::with_suffix(".bam").expect("tempfile");
    let sort_status = Command::new("samtools")
        .args([
            "sort",
            "-n",
            input.to_str().unwrap(),
            "-o",
            tmp.path().to_str().unwrap(),
        ])
        .status()
        .expect("spawn samtools sort -n");
    assert!(sort_status.success(), "samtools sort -n failed");

    let coord_out = run_ours(&input);
    let name_out = run_ours(tmp.path());

    // Compare checksum value lines only (not filenames).
    let coord_lines = data_lines(&coord_out);
    let name_lines = data_lines(&name_out);

    assert_eq!(
        coord_lines, name_lines,
        "checksum changed after name-sort:\n--- coord-sorted ---\n{coord_out}\n--- name-sorted ---\n{name_out}"
    );
}
