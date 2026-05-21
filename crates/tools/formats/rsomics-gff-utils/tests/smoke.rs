use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-gff-utils"))
}

fn gff() -> String {
    format!("{}/tests/golden/small.gff", env!("CARGO_MANIFEST_DIR"))
}

fn chrom_map() -> String {
    format!("{}/tests/golden/chrom_map.txt", env!("CARGO_MANIFEST_DIR"))
}

fn run_ok(cmd: &mut Command) -> String {
    let out = cmd.output().unwrap();
    assert!(
        out.status.success(),
        "FAILED: {:?}\nstderr: {}",
        cmd,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn count() {
    let s = run_ok(bin().arg("count").arg(gff()));
    assert_eq!(s.trim(), "13");
}

#[test]
fn chroms() {
    let s = run_ok(bin().arg("chroms").arg(gff()));
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines.contains(&"chr1"));
    assert!(lines.contains(&"chr2"));
}

#[test]
fn attributes() {
    let s = run_ok(bin().arg("attributes").arg(gff()));
    assert!(s.contains("ID"));
    assert!(s.contains("Parent"));
    assert!(s.contains("gene_name"));
}

#[test]
fn cds() {
    let s = run_ok(bin().arg("cds").arg(gff()));
    let lines: Vec<&str> = s.trim().lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(lines.len(), 3);
}

#[test]
fn exon_count() {
    let s = run_ok(bin().arg("exon-count").arg(gff()));
    assert_eq!(s.trim(), "5");
}

#[test]
fn exons() {
    let s = run_ok(bin().arg("exons").arg(gff()));
    let lines: Vec<&str> = s.trim().lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(lines.len(), 5);
}

#[test]
fn extract() {
    let s = run_ok(
        bin()
            .arg("extract")
            .arg(gff())
            .args(["-k", "gene_name", "--type", "gene"]),
    );
    assert!(s.contains("BRCA1"));
    assert!(s.contains("TP53"));
}

#[test]
fn features() {
    let s = run_ok(bin().arg("features").arg(gff()));
    assert!(s.contains("gene\t"));
    assert!(s.contains("exon\t"));
    assert!(s.contains("CDS\t"));
}

#[test]
fn filter_by_type() {
    let s = run_ok(bin().arg("filter").arg(gff()).args(["--type", "gene"]));
    let lines: Vec<&str> = s.trim().lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(lines.len(), 2);
}

#[test]
fn filter_by_pattern() {
    let s = run_ok(bin().arg("filter").arg(gff()).args(["-e", "BRCA1"]));
    let lines: Vec<&str> = s.trim().lines().filter(|l| !l.starts_with('#')).collect();
    assert!(lines.len() >= 1);
}

#[test]
fn gene_count() {
    let s = run_ok(bin().arg("gene-count").arg(gff()));
    assert_eq!(s.trim(), "2");
}

#[test]
fn genes() {
    let s = run_ok(bin().arg("genes").arg(gff()));
    let lines: Vec<&str> = s.trim().lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(lines.len(), 2);
    assert!(s.contains("gene1"));
    assert!(s.contains("gene2"));
}

#[test]
fn grep() {
    let s = run_ok(bin().arg("grep").arg(gff()).args(["-e", "BRCA1"]));
    let lines: Vec<&str> = s.trim().lines().filter(|l| !l.starts_with('#')).collect();
    assert!(lines.len() >= 1);
    assert!(s.contains("BRCA1"));
}

#[test]
fn grep_invert() {
    let s = run_ok(
        bin()
            .arg("grep")
            .arg(gff())
            .args(["-e", "BRCA1", "--invert"]),
    );
    assert!(!s.contains("BRCA1"));
}

#[test]
fn introns() {
    let s = run_ok(bin().arg("introns").arg(gff()));
    assert!(!s.is_empty());
}

#[test]
fn len() {
    let s = run_ok(bin().arg("len").arg(gff()));
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 13);
}

#[test]
fn parents() {
    let s = run_ok(bin().arg("parents").arg(gff()));
    assert!(s.contains("gene1") || s.contains("mrna1"));
}

#[test]
fn rename() {
    let s = run_ok(bin().arg("rename").arg(gff()).arg("-m").arg(chrom_map()));
    assert!(s.contains("chromosome1"));
    assert!(s.contains("chromosome2"));
    let data: Vec<&str> = s.lines().filter(|l| !l.starts_with('#')).collect();
    assert!(
        !data
            .iter()
            .any(|l| l.starts_with("chr1\t") || l.starts_with("chr2\t"))
    );
}

#[test]
fn sort() {
    let s = run_ok(bin().arg("sort").arg(gff()));
    let lines: Vec<&str> = s.trim().lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(lines.len(), 13);
}

#[test]
fn sources() {
    let s = run_ok(bin().arg("sources").arg(gff()));
    assert!(s.contains("ensembl"));
    assert!(s.contains("refseq"));
}

#[test]
fn stats() {
    let s = run_ok(bin().arg("stats").arg(gff()));
    assert!(s.contains("total\t13"));
    assert!(s.contains("chromosomes\t2"));
}

#[test]
fn strand_stats() {
    let s = run_ok(bin().arg("strand-stats").arg(gff()));
    assert!(s.contains('+'));
    assert!(s.contains('-'));
}

#[test]
fn subset() {
    let s = run_ok(bin().arg("subset").arg(gff()).args(["--type", "exon"]));
    let lines: Vec<&str> = s.trim().lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(lines.len(), 5);
}

#[test]
fn summary() {
    let s = run_ok(bin().arg("summary").arg(gff()));
    assert!(!s.is_empty());
}

#[test]
fn to_bed() {
    let s = run_ok(bin().arg("to-bed").arg(gff()));
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 13);
    assert!(lines[0].contains('\t'));
}

#[test]
fn transcripts() {
    let s = run_ok(bin().arg("transcripts").arg(gff()));
    let lines: Vec<&str> = s.trim().lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(lines.len(), 2);
}

#[test]
fn utr() {
    let s = run_ok(bin().arg("utr").arg(gff()));
    let lines: Vec<&str> = s.trim().lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(lines.len(), 1);
}

#[test]
fn validate() {
    let out = bin().arg("validate").arg(gff()).output().unwrap();
    assert!(out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("OK"));
}

#[test]
fn split() {
    let dir = std::env::temp_dir().join("rsomics-gff-split-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let prefix = dir.join("out_");
    let out = bin()
        .arg("split")
        .arg(gff())
        .arg("-o")
        .arg(&prefix)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let entries: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert!(entries.len() >= 2);
    let _ = std::fs::remove_dir_all(&dir);
}
