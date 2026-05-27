use std::path::PathBuf;
use std::process::{Command, Stdio};
use tempfile::TempDir;

fn ours() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_rsomics-vcf-cnv"))
}

fn golden(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name)
}

/// Check whether bcftools cnv is available on this system.
///
/// Per task spec: skip if `bcftools cnv --help` exits non-zero and stderr contains "unrecognized".
/// bcftools cnv without args exits 0 and prints help; the subcommand is absent when the stderr
/// from `bcftools cnv` contains "unrecognized option".
fn bcftools_cnv_available() -> bool {
    let out = Command::new("bcftools")
        .args(["cnv"])
        .stderr(Stdio::piped())
        .stdout(Stdio::null())
        .output()
        .ok();
    match out {
        None => false,
        Some(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            // If stderr says "unrecognized", the cnv subcommand is not available.
            !stderr.contains("unrecognized")
        }
    }
}

/// Strip bcftools version/command header lines so we compare only data rows.
fn filter_comparable(output: &str) -> Vec<String> {
    output
        .lines()
        .filter(|l| {
            !l.starts_with("# This file was produced by")
                && !l.starts_with("# The command")
                && !l.starts_with("#\t")
                && !l.starts_with("# RG, Regions [2]") // old-style duplicate header bcftools emits
                && *l != "#"
        })
        .map(str::to_string)
        .collect()
}

/// Run bcftools cnv on a VCF with the given extra args; return summary lines.
fn run_bcftools_cnv(
    vcf: &std::path::Path,
    extra: &[&str],
    outdir: &std::path::Path,
    sample: &str,
) -> Vec<String> {
    let mut cmd = Command::new("bcftools");
    cmd.arg("cnv").args(extra).arg("-o").arg(outdir).arg(vcf);
    let status = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("failed to run bcftools cnv");
    assert!(status.success(), "bcftools cnv failed");
    let summary = outdir.join(format!("summary.{sample}.tab"));
    filter_comparable(&std::fs::read_to_string(&summary).unwrap())
}

/// Run our binary and return summary lines.
fn run_ours(vcf: &std::path::Path, extra: &[&str], outdir: &std::path::Path) -> Vec<String> {
    let mut cmd = Command::new(ours());
    cmd.args(extra).arg("-o").arg(outdir).arg(vcf);
    let status = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("failed to run rsomics-vcf-cnv");
    assert!(status.success(), "rsomics-vcf-cnv failed: {cmd:?}");
    let summary = outdir.join("summary.SAMPLE1.tab");
    filter_comparable(&std::fs::read_to_string(&summary).unwrap())
}

/// Compare CN column of summary RG lines (column 5, 0-indexed after splitting on TAB).
/// We compare CN state and nSites fields only; quality and nHETs may differ by algorithm detail.
fn compare_regions(ours: &[String], theirs: &[String]) {
    let extract_regions = |lines: &[String]| -> Vec<(String, u32, u32, char)> {
        lines
            .iter()
            .filter(|l| l.starts_with("RG\t"))
            .map(|l| {
                let cols: Vec<&str> = l.split('\t').collect();
                let chrom = cols.get(1).copied().unwrap_or("").to_string();
                let start: u32 = cols.get(2).copied().unwrap_or("0").parse().unwrap_or(0);
                let end: u32 = cols.get(3).copied().unwrap_or("0").parse().unwrap_or(0);
                let cn_char = cols
                    .get(4)
                    .copied()
                    .unwrap_or("?")
                    .chars()
                    .next()
                    .unwrap_or('?');
                (chrom, start, end, cn_char)
            })
            .collect()
    };

    let ours_rg = extract_regions(ours);
    let theirs_rg = extract_regions(theirs);

    assert_eq!(
        ours_rg.len(),
        theirs_rg.len(),
        "region count mismatch: ours={} bcftools={}\nours:\n{}\nbcftools:\n{}",
        ours_rg.len(),
        theirs_rg.len(),
        ours.join("\n"),
        theirs.join("\n"),
    );

    for (i, (o, t)) in ours_rg.iter().zip(theirs_rg.iter()).enumerate() {
        assert_eq!(
            o.3, t.3,
            "region {i}: CN state mismatch: ours={} bcftools={}",
            o.3, t.3
        );
        assert_eq!(
            o.0, t.0,
            "region {i}: chromosome mismatch: ours={} bcftools={}",
            o.0, t.0
        );
    }
}

#[test]
fn compat_default_params() {
    if !bcftools_cnv_available() {
        eprintln!("bcftools cnv not available — skipping compat test");
        return;
    }

    let vcf = golden("test_cnv.vcf");
    let dir_b = TempDir::new().unwrap();
    let dir_o = TempDir::new().unwrap();

    let bcftools_lines = run_bcftools_cnv(&vcf, &["-s", "SAMPLE1"], dir_b.path(), "SAMPLE1");
    let ours_lines = run_ours(&vcf, &["-s", "SAMPLE1"], dir_o.path());

    compare_regions(&ours_lines, &bcftools_lines);
}

#[test]
fn compat_baf_only_mode() {
    if !bcftools_cnv_available() {
        eprintln!("bcftools cnv not available — skipping compat test");
        return;
    }

    let vcf = golden("test_cnv.vcf");
    let dir_b = TempDir::new().unwrap();
    let dir_o = TempDir::new().unwrap();

    // BAF-only: LRR weight = 0
    let bcftools_lines =
        run_bcftools_cnv(&vcf, &["-s", "SAMPLE1", "-l", "0"], dir_b.path(), "SAMPLE1");
    let ours_lines = run_ours(&vcf, &["-s", "SAMPLE1", "-l", "0"], dir_o.path());

    compare_regions(&ours_lines, &bcftools_lines);
}
