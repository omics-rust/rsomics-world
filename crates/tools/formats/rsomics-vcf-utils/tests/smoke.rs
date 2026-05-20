use std::path::Path;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-vcf-utils"))
}

fn fixture() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/small.vcf"
    ))
}

#[test]
fn count() {
    let out = bin().arg("count").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert_eq!(s.trim(), "5");
}

#[test]
fn chroms() {
    let out = bin().arg("chroms").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines, vec!["chr1", "chr2"]);
}

#[test]
fn samples() {
    let out = bin().arg("samples").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines, vec!["SAMPLE1", "SAMPLE2"]);
}

#[test]
fn snps() {
    let out = bin().arg("snps").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let records: Vec<&str> = s.lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(records.len(), 3);
}

#[test]
fn indels() {
    let out = bin().arg("indels").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let records: Vec<&str> = s.lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(records.len(), 2);
}

#[test]
fn pass() {
    let out = bin().arg("pass").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let records: Vec<&str> = s.lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(records.len(), 5);
}

#[test]
fn ts_tv() {
    let out = bin().arg("ts-tv").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("transitions\t3"));
    assert!(s.contains("transversions\t0"));
}

#[test]
fn type_counts() {
    let out = bin().arg("type-counts").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("SNPs\t3"));
    assert!(s.contains("indels\t2"));
}

#[test]
fn genotypes() {
    let out = bin().arg("genotypes").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("CHROM\tPOS\tSAMPLE1\tSAMPLE2"));
    let data_lines: Vec<&str> = s.lines().filter(|l| !l.starts_with("CHROM")).collect();
    assert_eq!(data_lines.len(), 5);
}

#[test]
fn het_hom() {
    let out = bin().arg("het-hom").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("sample\thet\thom_alt"));
    assert!(s.contains("SAMPLE1"));
    assert!(s.contains("SAMPLE2"));
}

#[test]
fn missing() {
    let out = bin().arg("missing").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("sample\tmissing\ttotal\tpct"));
    assert!(s.contains("SAMPLE1"));
}

#[test]
fn sort() {
    let out = bin().arg("sort").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let data: Vec<&str> = s.lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(data.len(), 5);
    assert!(data[0].starts_with("chr1\t100"));
    assert!(data[4].starts_with("chr2\t250"));
}

#[test]
fn view_count_only() {
    let out = bin()
        .args(["view", "--count-only"])
        .arg(fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert_eq!(s.trim(), "5");
}

#[test]
fn view_header_only() {
    let out = bin()
        .args(["view", "--header-only"])
        .arg(fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = s.lines().collect();
    assert!(lines.iter().all(|l| l.starts_with('#')));
    assert!(lines.len() >= 5);
}

#[test]
fn view_region() {
    let out = bin()
        .args(["view", "-r", "chr2"])
        .arg(fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let data: Vec<&str> = s.lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(data.len(), 2);
}

#[test]
fn info() {
    let out = bin()
        .args(["info", "-k", "DP"])
        .arg(fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("chr1\t100\t40"));
    assert!(s.contains("chr1\t200\t60"));
}

#[test]
fn maf() {
    let out = bin()
        .args(["maf", "--min-maf", "0.0", "--max-maf", "0.5"])
        .arg(fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
}

#[test]
fn af() {
    let out = bin().arg("af").arg(fixture()).output().unwrap();
    assert!(out.status.success());
}

#[test]
fn dp() {
    let out = bin().arg("dp").arg(fixture()).output().unwrap();
    assert!(out.status.success());
}

#[test]
fn per_chrom() {
    let out = bin().arg("per-chrom").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("chr1\t3"));
    assert!(s.contains("chr2\t2"));
}

#[test]
fn tail() {
    let out = bin()
        .args(["tail", "-n", "2"])
        .arg(fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let data: Vec<&str> = s.lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(data.len(), 2);
}

#[test]
fn to_tsv() {
    let out = bin().arg("to-tsv").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = s.lines().collect();
    assert_eq!(lines.len(), 6);
    assert!(lines[0].starts_with("CHROM\t"));
}

#[test]
fn grep() {
    let out = bin()
        .args(["grep", "-e", "chr1"])
        .arg(fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let data: Vec<&str> = s.lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(data.len(), 3);
}

#[test]
fn biallelic() {
    let out = bin().arg("biallelic").arg(fixture()).output().unwrap();
    assert!(out.status.success());
}

#[test]
fn qual_filter() {
    let out = bin()
        .args(["qual-filter", "--min-qual", "25"])
        .arg(fixture())
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let data: Vec<&str> = s.lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(data.len(), 3);
}
