use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bed-utils"))
}

fn golden(name: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), name)
}

fn bed() -> String {
    golden("small.bed")
}
fn genome() -> String {
    golden("genome.txt")
}
fn bed_b() -> String {
    golden("b.bed")
}
fn fa() -> String {
    golden("ref.fa")
}
fn gff() -> String {
    golden("features.gff")
}
fn chrom_map() -> String {
    golden("chrom_map.txt")
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

// ── single-input ops ──────────────────────────────────────────────

#[test]
fn count() {
    let s = run_ok(bin().arg("count").arg(bed()));
    assert_eq!(s.trim(), "6");
}

#[test]
fn chroms() {
    let s = run_ok(bin().arg("chroms").arg(bed()));
    let lines: Vec<&str> = s.trim().lines().collect();
    assert!(lines.contains(&"chr1"));
    assert!(lines.contains(&"chr2"));
    assert_eq!(lines.len(), 2);
}

#[test]
fn chroms_sizes() {
    let s = run_ok(bin().arg("chroms-sizes").arg(bed()));
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(s.contains("chr1\t"));
    assert!(s.contains("chr2\t"));
}

#[test]
fn coverage_hist() {
    let s = run_ok(bin().arg("coverage-hist").arg(bed()));
    assert!(!s.is_empty());
}

#[test]
fn flank_bp() {
    let s = run_ok(bin().args(["flank-bp", "-b", "5"]).arg(bed()));
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 6);
}

#[test]
fn len() {
    let s = run_ok(bin().arg("len").arg(bed()));
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 6);
    assert!(s.contains("20"));
    assert!(s.contains("30"));
    assert!(s.contains("40"));
}

#[test]
fn merge() {
    let s = run_ok(bin().args(["merge", "-i"]).arg(bed()));
    let lines: Vec<&str> = s.trim().lines().collect();
    assert!(lines.len() < 6);
}

#[test]
fn merge_by_name() {
    let s = run_ok(bin().arg("merge-by-name").arg(bed()));
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 5);
}

#[test]
fn merge_overlaps() {
    let s = run_ok(bin().arg("merge-overlaps").arg(bed()));
    let lines: Vec<&str> = s.trim().lines().collect();
    assert!(lines.len() < 6);
}

#[test]
fn midpoint() {
    let s = run_ok(bin().arg("midpoint").arg(bed()));
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 6);
}

#[test]
fn promoters() {
    let s = run_ok(
        bin()
            .args(["promoters", "-u", "100", "-d", "50"])
            .arg(bed()),
    );
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 6);
}

#[test]
fn resize() {
    let s = run_ok(bin().args(["resize", "-s", "10"]).arg(bed()));
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 6);
}

#[test]
fn sample() {
    let s = run_ok(bin().args(["sample", "-n", "3", "--seed", "42"]).arg(bed()));
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 3);
}

#[test]
fn sort() {
    let s = run_ok(bin().args(["sort", "-i"]).arg(bed()));
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 6);
    assert!(lines[0].starts_with("chr1\t"));
}

#[test]
fn sort_name() {
    let s = run_ok(bin().arg("sort-name").arg(bed()));
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 6);
}

#[test]
fn sort_size() {
    let s = run_ok(bin().arg("sort-size").arg(bed()));
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 6);
}

#[test]
fn spacing() {
    let s = run_ok(bin().arg("spacing").arg(bed()));
    assert!(!s.is_empty());
}

#[test]
fn stats() {
    let s = run_ok(bin().arg("stats").arg(bed()));
    assert!(s.contains("count\t6"));
    assert!(s.contains("total_bases\t170"));
    assert!(s.contains("min_len\t20"));
    assert!(s.contains("max_len\t40"));
}

#[test]
fn tail() {
    let s = run_ok(bin().args(["tail", "-n", "2"]).arg(bed()));
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 2);
}

#[test]
fn to_gff() {
    let s = run_ok(bin().arg("to-gff").arg(bed()));
    let data: Vec<&str> = s.trim().lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(data.len(), 6);
}

#[test]
fn to_gff3() {
    let s = run_ok(bin().arg("to-gff3").arg(bed()));
    let data: Vec<&str> = s.trim().lines().filter(|l| !l.starts_with('#')).collect();
    assert_eq!(data.len(), 6);
}

#[test]
fn to_igv() {
    let s = run_ok(bin().arg("to-igv").arg(bed()));
    assert!(!s.is_empty());
}

#[test]
fn to_wig() {
    let s = run_ok(bin().arg("to-wig").arg(bed()));
    assert!(!s.is_empty());
}

#[test]
fn total_bp() {
    let s = run_ok(bin().arg("total-bp").arg(bed()));
    assert_eq!(s.trim(), "170");
}

#[test]
fn total_span() {
    let s = run_ok(bin().arg("total-span").arg(bed()));
    assert!(s.contains("chr1\t"));
    assert!(s.contains("chr2\t"));
}

#[test]
fn unique() {
    let out = bin().arg("unique").arg(bed()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 5);
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("5 unique"));
}

#[test]
fn validate() {
    let out = bin().arg("validate").arg(bed()).output().unwrap();
    assert!(out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("OK"));
}

// ── genome-file ops ───────────────────────────────────────────────

#[test]
fn complement() {
    let s = run_ok(
        bin()
            .args(["complement", "-i"])
            .arg(bed())
            .arg("-g")
            .arg(genome()),
    );
    assert!(!s.is_empty());
}

#[test]
fn flank() {
    let s = run_ok(
        bin()
            .arg("flank")
            .arg(bed())
            .arg("-g")
            .arg(genome())
            .args(["-l", "10", "-r", "10"]),
    );
    let lines: Vec<&str> = s.trim().lines().collect();
    assert!(lines.len() >= 6);
}

#[test]
fn genomecov() {
    let s = run_ok(bin().arg("genomecov").arg(bed()).arg("-g").arg(genome()));
    assert!(!s.is_empty());
}

#[test]
fn makewindows() {
    let s = run_ok(
        bin()
            .args(["makewindows", "-g"])
            .arg(genome())
            .args(["-w", "100"]),
    );
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 15);
}

#[test]
fn random() {
    let s = run_ok(
        bin()
            .args(["random", "-g"])
            .arg(genome())
            .args(["-n", "5", "-l", "50", "--seed", "42"]),
    );
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 5);
}

#[test]
fn shift() {
    let s = run_ok(
        bin()
            .arg("shift")
            .arg(bed())
            .arg("-g")
            .arg(genome())
            .args(["-s", "10"]),
    );
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 6);
}

#[test]
fn slop() {
    let s = run_ok(
        bin()
            .arg("slop")
            .arg(bed())
            .arg("-g")
            .arg(genome())
            .args(["-l", "5", "-r", "5"]),
    );
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 6);
}

// ── two-BED ops ───────────────────────────────────────────────────

#[test]
fn closest() {
    let s = run_ok(bin().arg("closest").arg(bed()).arg(bed_b()));
    assert!(!s.is_empty());
}

#[test]
fn intersect() {
    let s = run_ok(
        bin()
            .args(["intersect", "-a"])
            .arg(bed())
            .arg("-b")
            .arg(bed_b()),
    );
    let lines: Vec<&str> = s.trim().lines().collect();
    assert!(lines.len() >= 2);
}

#[test]
fn jaccard() {
    let s = run_ok(bin().arg("jaccard").arg(bed()).arg(bed_b()));
    assert!(s.contains("jaccard\t"));
    assert!(s.contains("intersection\t"));
}

#[test]
fn overlap() {
    let s = run_ok(bin().arg("overlap").arg(bed()).arg(bed_b()));
    assert!(s.contains("overlap") || s.contains("intersection") || s.contains('{'));
}

#[test]
fn subtract() {
    let s = run_ok(
        bin()
            .args(["subtract", "-a"])
            .arg(bed())
            .arg("-b")
            .arg(bed_b()),
    );
    assert!(!s.is_empty());
}

#[test]
fn union() {
    let s = run_ok(bin().arg("union").arg(bed()).arg(bed_b()));
    let lines: Vec<&str> = s.trim().lines().collect();
    assert!(lines.len() >= 4);
}

#[test]
fn window() {
    let s = run_ok(
        bin()
            .arg("window")
            .arg(bed())
            .arg(bed_b())
            .args(["-w", "500"]),
    );
    assert!(!s.is_empty());
}

#[test]
fn map() {
    let s =
        run_ok(
            bin()
                .arg("map")
                .arg(bed())
                .arg(bed_b())
                .args(["--operation", "mean", "-c", "5"]),
        );
    assert!(!s.is_empty());
}

// ── BED + FASTA ops ───────────────────────────────────────────────

#[test]
fn getfasta() {
    // getfasta requires a .fai index; skip if samtools not available
    let idx = format!("{}.fai", fa());
    if !std::path::Path::new(&idx).exists() {
        let _ = std::process::Command::new("samtools")
            .args(["faidx", &fa()])
            .status();
    }
    if !std::path::Path::new(&idx).exists() {
        eprintln!("skipping getfasta: no .fai index");
        return;
    }
    let s = run_ok(bin().arg("getfasta").arg(bed()).arg("-f").arg(fa()));
    assert!(s.contains('>'));
}

#[test]
fn nuc() {
    let s = run_ok(bin().arg("nuc").arg(bed()).arg("-f").arg(fa()));
    assert!(!s.is_empty());
}

#[test]
fn to_fasta() {
    let s = run_ok(bin().arg("to-fasta").arg(bed()).arg("-f").arg(fa()));
    assert!(s.contains('>'));
}

// ── BED + GFF ─────────────────────────────────────────────────────

#[test]
fn annotate() {
    let s = run_ok(bin().arg("annotate").arg(bed()).arg(gff()).args([
        "--type",
        "gene",
        "--attribute",
        "gene_name",
    ]));
    assert!(!s.is_empty());
}

// ── BED + chrom map ───────────────────────────────────────────────

#[test]
fn rename() {
    let s = run_ok(bin().arg("rename").arg(bed()).arg("-m").arg(chrom_map()));
    assert!(s.contains("chromosome1"));
    assert!(s.contains("chromosome2"));
    assert!(!s.contains("chr1"));
}
