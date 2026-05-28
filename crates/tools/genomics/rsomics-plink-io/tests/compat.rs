use std::path::PathBuf;
use std::process::{Command, Stdio};

fn ours() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_rsomics-plink-io"))
}

fn bfile() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/small")
}

fn plink_available() -> bool {
    Command::new("plink")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[test]
fn freq_runs_successfully() {
    let out = Command::new(ours())
        .args(["freq", bfile().to_str().unwrap()])
        .output()
        .expect("rsomics-plink-io freq");
    assert!(
        out.status.success(),
        "freq failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("CHR\tSNP\tA1\tA2\tMAF\tNCHROBS"),
        "missing header"
    );
    assert!(stdout.contains("chr"), "missing chrom column");
}

#[test]
fn missing_runs_successfully() {
    let out = Command::new(ours())
        .args(["missing", bfile().to_str().unwrap()])
        .output()
        .expect("rsomics-plink-io missing");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("CHR\tSNP\tN_MISS\tN_GENO\tF_MISS"),
        "missing header"
    );
}

#[test]
fn hardy_runs_successfully() {
    let out = Command::new(ours())
        .args(["hardy", bfile().to_str().unwrap()])
        .output()
        .expect("rsomics-plink-io hardy");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("CHR\tSNP\tTEST\tGENO"), "missing header");
}

#[test]
fn to_vcf_runs_successfully() {
    let out = Command::new(ours())
        .args(["to-vcf", bfile().to_str().unwrap()])
        .output()
        .expect("rsomics-plink-io to-vcf");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.starts_with("##fileformat=VCF"), "VCF header missing");
}

#[test]
fn to_012_runs_successfully() {
    let out = Command::new(ours())
        .args(["to-012", bfile().to_str().unwrap()])
        .output()
        .expect("rsomics-plink-io to-012");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.is_empty(), "no output produced");
}

#[test]
fn exit_nonzero_on_missing_file() {
    let status = Command::new(ours())
        .args(["freq", "/nonexistent/path/fixture"])
        .status()
        .expect("rsomics-plink-io freq missing");
    assert!(!status.success());
}

#[test]
fn freq_variant_count() {
    let out = Command::new(ours())
        .args(["freq", bfile().to_str().unwrap()])
        .output()
        .expect("rsomics-plink-io freq");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // 100 variants + 1 header line
    let data_lines = stdout.lines().filter(|l| !l.starts_with("CHR")).count();
    assert_eq!(data_lines, 100, "expected 100 data lines, got {data_lines}");
}

#[test]
fn compat_plink_freq() {
    if !plink_available() {
        eprintln!("plink not available — skipping compat test");
        return;
    }

    let out_ours = Command::new(ours())
        .args(["freq", bfile().to_str().unwrap()])
        .output()
        .expect("rsomics-plink-io freq");
    assert!(out_ours.status.success());

    // plink --bfile <prefix> --freq --out /dev/stdout
    // plink output format differs slightly; we compare SNP IDs match.
    let tmp = tempfile::tempdir().unwrap();
    let plink_out = tmp.path().join("out");
    let status = Command::new("plink")
        .args([
            "--bfile",
            bfile().to_str().unwrap(),
            "--freq",
            "--out",
            plink_out.to_str().unwrap(),
            "--silent",
        ])
        .status()
        .expect("plink freq");
    assert!(status.success());

    let plink_freq_file = tmp.path().join("out.frq");
    if plink_freq_file.exists() {
        let plink_text = std::fs::read_to_string(&plink_freq_file).unwrap();
        let ours_text = String::from_utf8_lossy(&out_ours.stdout);
        // Compare SNP IDs: both should list the same variants
        let plink_snps: std::collections::HashSet<&str> = plink_text
            .lines()
            .skip(1)
            .filter_map(|l| l.split_whitespace().nth(1))
            .collect();
        let our_snps: std::collections::HashSet<&str> = ours_text
            .lines()
            .skip(1)
            .filter_map(|l| l.split('\t').nth(1))
            .collect();
        assert_eq!(
            plink_snps, our_snps,
            "SNP lists differ between plink and rsomics-plink-io"
        );
    }
}
