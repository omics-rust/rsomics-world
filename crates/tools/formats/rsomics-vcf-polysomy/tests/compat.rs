use std::path::PathBuf;
use std::process::{Command, Stdio};
use tempfile::TempDir;

fn ours() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_rsomics-vcf-polysomy"))
}

fn bcftools_version() -> Option<String> {
    let out = Command::new("bcftools").arg("--version").output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let first = text.lines().next()?.to_string();
    Some(first)
}

fn bcftools_has_polysomy() -> bool {
    // bcftools polysomy requires GSL; not all builds include it.
    Command::new("bcftools")
        .args(["polysomy", "--help"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.code() != Some(127))
        .unwrap_or(false)
}

/// VCF used for basic smoke tests (existence of output, bcftools comparison).
/// Uses a 3-cluster BAF pattern that both tools classify as CN=-1 (too sparse to fit).
fn write_test_vcf(dir: &TempDir) -> PathBuf {
    let path = dir.path().join("test.vcf");
    let mut lines = vec![
        "##fileformat=VCFv4.2".to_string(),
        "##FILTER=<ID=PASS,Description=\"All filters passed\">".to_string(),
        "##contig=<ID=chr1,length=248956422>".to_string(),
        "##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">".to_string(),
        "##FORMAT=<ID=BAF,Number=1,Type=Float,Description=\"B-allele frequency\">".to_string(),
        "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tsample1".to_string(),
    ];
    let mut pos = 1000u32;
    let baf_values: &[f32] = &[
        // RR cluster (near 0)
        0.02, 0.03, 0.01, 0.04, 0.02, 0.05, 0.03, 0.02, 0.01, 0.03, 0.04, 0.02, 0.03, 0.05, 0.02,
        0.03, 0.01, 0.04, 0.02, 0.03, 0.04, 0.02, 0.03, 0.01, 0.05, 0.02, 0.03, 0.04, 0.01, 0.02,
        // RA cluster (near 0.5)
        0.48, 0.50, 0.52, 0.49, 0.51, 0.50, 0.48, 0.52, 0.50, 0.49, 0.51, 0.50, 0.52, 0.48, 0.50,
        0.51, 0.49, 0.50, 0.48, 0.52, 0.50, 0.49, 0.51, 0.50, 0.48, 0.52, 0.50, 0.49, 0.51, 0.50,
        // AA cluster (near 1.0)
        0.97, 0.98, 0.96, 0.99, 0.97, 0.98, 0.96, 0.97, 0.99, 0.98, 0.97, 0.96, 0.98, 0.99, 0.97,
        0.98, 0.96, 0.97, 0.99, 0.98, 0.97, 0.96, 0.98, 0.99, 0.97, 0.98, 0.96, 0.97, 0.99, 0.98,
    ];
    for &baf in baf_values {
        lines.push(format!(
            "chr1\t{pos}\t.\tA\tG\t50\tPASS\t.\tGT:BAF\t0/1:{baf:.2}"
        ));
        pos += 1000;
    }
    std::fs::write(&path, lines.join("\n") + "\n").expect("write test VCF");
    path
}

/// Path to the golden diploid fixture (500 BAF values, Normal(0.5, 0.05), seed 12345).
/// Verified to yield CN=2 in both bcftools polysomy 1.23.1 and our LM solver.
fn diploid_fixture_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/diploid_ra_500.vcf")
}

/// Extract CN lines from a dist.dat file, stripping header comments.
fn parse_cn_lines(dist_dat: &str) -> Vec<String> {
    dist_dat
        .lines()
        .filter(|l| l.starts_with("CN\t"))
        .map(str::to_string)
        .collect()
}

/// Extract just the chromosome and rounded copy-number from a CN line.
/// Format: "CN\t<chrom>\t<cn_float>\t<fit>"
/// We compare chrom + CN rounded to nearest integer for robustness
/// across different numerical solvers (our LM vs bcftools' GSL/LM).
fn cn_integer_summary(line: &str) -> String {
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() < 3 {
        return line.to_string();
    }
    let chrom = parts[1];
    let cn_rounded = parts[2]
        .parse::<f64>()
        .map(|v| v.round() as i64)
        .unwrap_or(-999);
    format!("CN\t{chrom}\t{cn_rounded}")
}

#[test]
fn ours_produces_dist_dat() {
    let dir = TempDir::new().unwrap();
    let vcf = write_test_vcf(&dir);
    let outdir = dir.path().join("out");

    let result = Command::new(ours())
        .args(["-o", outdir.to_str().unwrap()])
        .arg(&vcf)
        .output()
        .expect("spawn rsomics-vcf-polysomy");

    assert!(
        result.status.success(),
        "rsomics-vcf-polysomy exited non-zero:\nstderr: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    let dat = outdir.join("dist.dat");
    assert!(dat.exists(), "dist.dat not created at {}", dat.display());

    let contents = std::fs::read_to_string(&dat).unwrap();
    assert!(contents.contains("DIST\t"), "dist.dat has no DIST lines");
    assert!(contents.contains("CN\t"), "dist.dat has no CN lines");
    assert!(contents.contains("FIT\t"), "dist.dat has no FIT lines");
}

#[test]
fn diploid_fixture_calls_cn2() {
    let dir = TempDir::new().unwrap();
    let vcf = diploid_fixture_path();
    let outdir = dir.path().join("out2");

    let result = Command::new(ours())
        .args(["-o", outdir.to_str().unwrap()])
        .arg(&vcf)
        .output()
        .expect("spawn rsomics-vcf-polysomy");

    assert!(
        result.status.success(),
        "rsomics-vcf-polysomy failed:\nstderr: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    let dat = std::fs::read_to_string(outdir.join("dist.dat")).unwrap();
    let cn_lines = parse_cn_lines(&dat);
    assert!(!cn_lines.is_empty(), "no CN lines in dist.dat");

    for line in &cn_lines {
        let summary = cn_integer_summary(line);
        // A diploid BAF distribution should yield CN=2.
        assert!(
            summary.ends_with("\t2"),
            "expected CN=2 for diploid fixture, got: {line}"
        );
    }
}

#[test]
fn cn_agrees_with_bcftools_polysomy() {
    let ver = match bcftools_version() {
        Some(v) => v,
        None => {
            eprintln!("skipping: bcftools not found");
            return;
        }
    };
    if !bcftools_has_polysomy() {
        eprintln!("skipping: bcftools polysomy unavailable (no GSL build): {ver}");
        return;
    }
    // Guard: only run against known-good version to avoid format drift.
    if !ver.contains("1.") {
        eprintln!("skipping: unexpected bcftools version: {ver}");
        return;
    }

    let dir = TempDir::new().unwrap();
    // Use the golden fixture that yields CN=2 in both tools, exercising the
    // full GMM fitting path (not just the trivial CN=-1 branch).
    let vcf = diploid_fixture_path();

    let our_outdir = dir.path().join("ours");
    let bcf_outdir = dir.path().join("bcf");

    // Run ours.
    let our_status = Command::new(ours())
        .args(["-o", our_outdir.to_str().unwrap()])
        .arg(&vcf)
        .output()
        .expect("spawn rsomics-vcf-polysomy");
    assert!(
        our_status.status.success(),
        "rsomics-vcf-polysomy failed:\nstderr: {}",
        String::from_utf8_lossy(&our_status.stderr)
    );

    // Run bcftools polysomy.
    let bcf_status = Command::new("bcftools")
        .args(["polysomy", "-o", bcf_outdir.to_str().unwrap()])
        .arg(&vcf)
        .output()
        .expect("spawn bcftools polysomy");
    assert!(
        bcf_status.status.success(),
        "bcftools polysomy failed:\nstderr: {}",
        String::from_utf8_lossy(&bcf_status.stderr)
    );

    // Compare CN calls (rounded to integer — solvers differ numerically).
    let our_dat = std::fs::read_to_string(our_outdir.join("dist.dat")).unwrap();
    let bcf_dat = std::fs::read_to_string(bcf_outdir.join("dist.dat")).unwrap();

    let our_cn: Vec<String> = parse_cn_lines(&our_dat)
        .into_iter()
        .map(|l| cn_integer_summary(&l))
        .collect();
    let bcf_cn: Vec<String> = parse_cn_lines(&bcf_dat)
        .into_iter()
        .map(|l| cn_integer_summary(&l))
        .collect();

    assert_eq!(
        our_cn, bcf_cn,
        "CN calls disagree (integer-rounded):\nOURS: {our_cn:?}\nBCFTOOLS: {bcf_cn:?}"
    );
}
