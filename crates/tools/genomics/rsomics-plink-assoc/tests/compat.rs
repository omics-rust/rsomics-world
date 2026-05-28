use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn ours() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_rsomics-plink-assoc"))
}

fn bfile() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/small")
}

fn quant_bfile() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/quant")
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
fn assoc_runs_successfully() {
    let out = Command::new(ours())
        .args(["assoc", bfile().to_str().unwrap()])
        .output()
        .expect("rsomics-plink-assoc assoc");
    assert!(
        out.status.success(),
        "assoc failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("CHR\tSNP\tBP\tA1\tF_A\tF_U\tA2\tCHISQ\tP\tOR"),
        "missing header"
    );
}

#[test]
fn assoc_variant_count() {
    let out = Command::new(ours())
        .args(["assoc", bfile().to_str().unwrap()])
        .output()
        .expect("rsomics-plink-assoc assoc");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // 100 variants + 1 header line
    let data_lines = stdout.lines().filter(|l| !l.starts_with("CHR")).count();
    assert_eq!(data_lines, 100, "expected 100 variants, got {data_lines}");
}

#[test]
fn assoc_p_values_in_range() {
    let out = Command::new(ours())
        .args(["assoc", bfile().to_str().unwrap()])
        .output()
        .expect("rsomics-plink-assoc assoc");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    for line in stdout.lines().skip(1) {
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 9 {
            continue;
        }
        let p: f64 = cols[8].parse().unwrap_or(-1.0);
        assert!(
            p.is_nan() || (0.0..=1.0).contains(&p),
            "p value out of range: {p} in line: {line}"
        );
    }
}

#[test]
fn linear_runs_successfully() {
    let out = Command::new(ours())
        .args(["linear", quant_bfile().to_str().unwrap()])
        .output()
        .expect("rsomics-plink-assoc linear");
    assert!(
        out.status.success(),
        "linear failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("CHR\tSNP\tBP\tNMISS\tA1\tBETA\tSE\tSTAT\tP"),
        "missing header"
    );
}

#[test]
fn linear_variant_count() {
    let out = Command::new(ours())
        .args(["linear", quant_bfile().to_str().unwrap()])
        .output()
        .expect("rsomics-plink-assoc linear");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let data_lines = stdout.lines().filter(|l| !l.starts_with("CHR")).count();
    assert_eq!(data_lines, 100, "expected 100 variants, got {data_lines}");
}

#[test]
fn exit_nonzero_on_missing_file() {
    let status = Command::new(ours())
        .args(["assoc", "/nonexistent/path/fixture"])
        .status()
        .expect("rsomics-plink-assoc assoc missing");
    assert!(!status.success());
}

#[test]
fn compat_plink_assoc() {
    if !plink_available() {
        eprintln!("plink not available — skipping compat test");
        return;
    }

    let out_ours = Command::new(ours())
        .args(["assoc", bfile().to_str().unwrap()])
        .output()
        .expect("rsomics-plink-assoc assoc");
    assert!(out_ours.status.success());

    let tmp = tempfile::tempdir().unwrap();
    let plink_out = tmp.path().join("out");
    let status = Command::new("plink")
        .args([
            "--bfile",
            bfile().to_str().unwrap(),
            "--assoc",
            "--out",
            plink_out.to_str().unwrap(),
            "--silent",
        ])
        .status()
        .expect("plink --assoc");
    assert!(status.success());

    let assoc_file = tmp.path().join("out.assoc");
    if assoc_file.exists() {
        let plink_text = std::fs::read_to_string(&assoc_file).unwrap();
        let ours_text = String::from_utf8_lossy(&out_ours.stdout);

        // Compare SNP IDs
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
            "SNP lists differ between plink and rsomics-plink-assoc"
        );
    }
}
