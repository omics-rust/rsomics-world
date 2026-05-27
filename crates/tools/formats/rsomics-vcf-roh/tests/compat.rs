use std::path::PathBuf;
use std::process::{Command, Stdio};
use tempfile::TempDir;

fn ours() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_rsomics-vcf-roh"))
}

fn bcftools_available() -> bool {
    Command::new("bcftools")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn write_test_vcf(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, content).expect("write vcf");
    path
}

/// Strip bcftools-version-specific header lines from the output so we can compare
/// the data content only. Both bcftools roh and our tool emit a `# RG ...` and/or
/// `# ST ...` column-legend header; bcftools also emits a `# This file was produced by`
/// preamble we must skip.
fn filter_comparable(output: &str) -> Vec<String> {
    output
        .lines()
        .filter(|l| {
            !l.starts_with("# This file was produced by")
                && !l.starts_with("# The command")
                && !l.starts_with("#\t")
                && l != &"#"
                && !l.starts_with("# RG\t[")
                && !l.starts_with("# ST\t[")
        })
        .map(str::to_string)
        .collect()
}

/// A synthetic VCF with AC/AN in INFO and PL in FORMAT. The sample `NA12878` has
/// a long run of hom-alt (autozygous-like) genotypes that the HMM should detect as ROH.
const ROH_VCF: &str = "\
##fileformat=VCFv4.2
##FILTER=<ID=PASS,Description=\"All filters passed\">
##contig=<ID=chr1,length=248956422>
##INFO=<ID=AC,Number=A,Type=Integer,Description=\"Allele count\">
##INFO=<ID=AN,Number=1,Type=Integer,Description=\"Total allele count\">
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">
##FORMAT=<ID=PL,Number=G,Type=Integer,Description=\"Phred-scaled genotype likelihoods\">
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tNA12878
chr1\t100\t.\tA\tG\t50\tPASS\tAC=1;AN=2\tGT:PL\t0/0:0,30,100
chr1\t200\t.\tC\tT\t50\tPASS\tAC=1;AN=2\tGT:PL\t1/1:100,30,0
chr1\t300\t.\tG\tA\t50\tPASS\tAC=1;AN=2\tGT:PL\t1/1:100,30,0
chr1\t400\t.\tT\tC\t50\tPASS\tAC=1;AN=2\tGT:PL\t1/1:100,30,0
chr1\t500\t.\tA\tT\t50\tPASS\tAC=1;AN=2\tGT:PL\t1/1:100,30,0
chr1\t600\t.\tC\tG\t50\tPASS\tAC=1;AN=2\tGT:PL\t1/1:100,30,0
chr1\t700\t.\tG\tC\t50\tPASS\tAC=1;AN=2\tGT:PL\t1/1:100,30,0
chr1\t800\t.\tT\tA\t50\tPASS\tAC=1;AN=2\tGT:PL\t1/1:100,30,0
chr1\t900\t.\tA\tG\t50\tPASS\tAC=1;AN=2\tGT:PL\t1/1:100,30,0
chr1\t1000\t.\tC\tT\t50\tPASS\tAC=1;AN=2\tGT:PL\t0/0:0,30,100
";

/// GT-only test VCF (for -G30 mode).
const GT_ONLY_VCF: &str = "\
##fileformat=VCFv4.2
##FILTER=<ID=PASS,Description=\"All filters passed\">
##contig=<ID=chr1,length=248956422>
##INFO=<ID=AC,Number=A,Type=Integer,Description=\"Allele count\">
##INFO=<ID=AN,Number=1,Type=Integer,Description=\"Total allele count\">
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tNA12878
chr1\t100\t.\tA\tG\t50\tPASS\tAC=1;AN=2\tGT\t0/0
chr1\t200\t.\tC\tT\t50\tPASS\tAC=1;AN=2\tGT\t1/1
chr1\t300\t.\tG\tA\t50\tPASS\tAC=1;AN=2\tGT\t1/1
chr1\t400\t.\tT\tC\t50\tPASS\tAC=1;AN=2\tGT\t1/1
chr1\t500\t.\tA\tT\t50\tPASS\tAC=1;AN=2\tGT\t1/1
chr1\t600\t.\tC\tG\t50\tPASS\tAC=1;AN=2\tGT\t1/1
chr1\t700\t.\tG\tC\t50\tPASS\tAC=1;AN=2\tGT\t1/1
chr1\t800\t.\tT\tA\t50\tPASS\tAC=1;AN=2\tGT\t1/1
chr1\t900\t.\tA\tG\t50\tPASS\tAC=1;AN=2\tGT\t1/1
chr1\t1000\t.\tC\tT\t50\tPASS\tAC=1;AN=2\tGT\t0/0
";

#[test]
fn compat_pl_mode_rg_output() {
    if !bcftools_available() {
        eprintln!("bcftools not available — skipping compat test");
        return;
    }

    let dir = TempDir::new().unwrap();
    let vcf = write_test_vcf(&dir, "roh.vcf", ROH_VCF);

    // --- bcftools reference ---
    let ref_out = Command::new("bcftools")
        .args(["roh", "-Or"])
        .arg(&vcf)
        .output()
        .expect("bcftools roh");
    assert!(
        ref_out.status.success(),
        "bcftools roh failed: {}",
        String::from_utf8_lossy(&ref_out.stderr)
    );
    let ref_stdout = String::from_utf8_lossy(&ref_out.stdout);
    let ref_lines = filter_comparable(&ref_stdout);

    // --- ours ---
    let our_out = Command::new(ours())
        .args(["-Or"])
        .arg(&vcf)
        .output()
        .expect("rsomics-vcf-roh");
    assert!(
        our_out.status.success(),
        "rsomics-vcf-roh failed: {}",
        String::from_utf8_lossy(&our_out.stderr)
    );
    let our_stdout = String::from_utf8_lossy(&our_out.stdout);
    let our_lines = filter_comparable(&our_stdout);

    // Compare RG lines: same sample, chrom, start, end, nmarkers.
    // Quality (Phred) can differ slightly due to floating-point differences in fwd-bwd, so
    // we compare integer-stable fields only.
    let ref_rg: Vec<Vec<&str>> = ref_lines
        .iter()
        .filter(|l| l.starts_with("RG\t"))
        .map(|l| l.split('\t').collect())
        .collect();

    let our_rg: Vec<Vec<&str>> = our_lines
        .iter()
        .filter(|l| l.starts_with("RG\t"))
        .map(|l| l.split('\t').collect())
        .collect();

    assert_eq!(
        ref_rg.len(),
        our_rg.len(),
        "RG line count differs. bcftools:\n{ref_stdout}\nours:\n{our_stdout}"
    );

    for (r, o) in ref_rg.iter().zip(our_rg.iter()) {
        // Fields: RG, sample, chrom, start, end, length, nmarkers, quality
        assert_eq!(r.get(1), o.get(1), "sample mismatch");
        assert_eq!(r.get(2), o.get(2), "chrom mismatch");
        assert_eq!(r.get(3), o.get(3), "start mismatch");
        assert_eq!(r.get(4), o.get(4), "end mismatch");
        assert_eq!(r.get(5), o.get(5), "length mismatch");
        assert_eq!(r.get(6), o.get(6), "nmarkers mismatch");
        // Quality (field 7) compared as rounded integer — tiny FP differences are acceptable.
        let rq: f64 = r.get(7).and_then(|v| v.parse().ok()).unwrap_or(0.0);
        let oq: f64 = o.get(7).and_then(|v| v.parse().ok()).unwrap_or(0.0);
        assert!(
            (rq - oq).abs() < 1.0,
            "quality differs too much: bcftools={rq} ours={oq}"
        );
    }
}

#[test]
fn compat_gt_only_mode_st_output() {
    if !bcftools_available() {
        eprintln!("bcftools not available — skipping compat test");
        return;
    }

    let dir = TempDir::new().unwrap();
    let vcf = write_test_vcf(&dir, "gt.vcf", GT_ONLY_VCF);

    // bcftools roh -G30 --AF-dflt 0.4 -Os
    let ref_out = Command::new("bcftools")
        .args(["roh", "-G30", "--AF-dflt", "0.4", "-Os"])
        .arg(&vcf)
        .output()
        .expect("bcftools roh");
    assert!(
        ref_out.status.success(),
        "bcftools roh (GT mode) failed: {}",
        String::from_utf8_lossy(&ref_out.stderr)
    );

    let our_out = Command::new(ours())
        .args(["-G30", "--AF-dflt", "0.4", "-Os"])
        .arg(&vcf)
        .output()
        .expect("rsomics-vcf-roh (GT mode)");
    assert!(
        our_out.status.success(),
        "rsomics-vcf-roh (GT mode) failed: {}",
        String::from_utf8_lossy(&our_out.stderr)
    );

    let ref_stdout = String::from_utf8_lossy(&ref_out.stdout);
    let our_stdout = String::from_utf8_lossy(&our_out.stdout);

    // Compare ST lines: sample, chrom, pos, state must match exactly.
    let ref_st: Vec<Vec<String>> = ref_stdout
        .lines()
        .filter(|l| l.starts_with("ST\t"))
        .map(|l| l.split('\t').map(str::to_string).collect())
        .collect();

    let our_st: Vec<Vec<String>> = our_stdout
        .lines()
        .filter(|l| l.starts_with("ST\t"))
        .map(|l| l.split('\t').map(str::to_string).collect())
        .collect();

    assert_eq!(
        ref_st.len(),
        our_st.len(),
        "ST line count differs.\nbcftools:\n{ref_stdout}\nours:\n{our_stdout}"
    );

    for (r, o) in ref_st.iter().zip(our_st.iter()) {
        // ST, sample, chrom, pos, state, quality
        assert_eq!(r.get(1), o.get(1), "sample mismatch in ST");
        assert_eq!(r.get(2), o.get(2), "chrom mismatch in ST");
        assert_eq!(r.get(3), o.get(3), "pos mismatch in ST");
        assert_eq!(r.get(4), o.get(4), "state mismatch in ST: bcftools={:?} ours={:?}", r.get(4), o.get(4));
    }
}

#[test]
fn exit_code_success_on_valid_input() {
    let dir = TempDir::new().unwrap();
    let vcf = write_test_vcf(&dir, "basic.vcf", ROH_VCF);

    let status = Command::new(ours())
        .args(["-G30", "--AF-dflt", "0.4", "-Or"])
        .arg(&vcf)
        .status()
        .expect("run rsomics-vcf-roh");
    assert!(status.success(), "expected exit 0, got: {status}");
}

#[test]
fn exit_code_nonzero_on_missing_file() {
    let status = Command::new(ours())
        .args(["-G30", "--AF-dflt", "0.4", "/nonexistent/path/to.vcf"])
        .status()
        .expect("run rsomics-vcf-roh");
    assert!(!status.success(), "expected non-zero exit for missing file");
}
