use std::path::PathBuf;
use std::process::Command;

fn golden(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name)
}

fn bin() -> String {
    env!("CARGO_BIN_EXE_rsomics-vcf-popgen").to_string()
}

fn run(args: &[&str]) -> String {
    let out = Command::new(bin())
        .args(args)
        .output()
        .expect("run rsomics-vcf-popgen");
    assert!(
        out.status.success(),
        "non-zero exit:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

// ─── freq ────────────────────────────────────────────────────────────────────

#[test]
fn freq_has_header() {
    let vcf = golden("small.vcf");
    let out = run(&["freq", vcf.to_str().unwrap()]);
    assert!(out.starts_with("CHROM\tPOS\tN_ALLELES\tN_CHR"));
}

#[test]
fn freq_counts_correct() {
    let vcf = golden("small.vcf");
    let out = run(&["freq", vcf.to_str().unwrap()]);
    // chr1:100 has 5 samples, all called. alt (G) count = 4 (0/1 + 1/1 + 0/1 = 1+2+1 = 4)
    // n_chr = 10, n_alt = 4, freq_alt = 0.4000
    let lines: Vec<&str> = out.lines().collect();
    let site100 = lines
        .iter()
        .find(|l| l.contains("chr1\t100\t"))
        .expect("site chr1:100");
    assert!(site100.contains("G:0.4000"), "alt freq at 100: {site100}");
}

#[test]
fn freq_skips_monomorphic() {
    let vcf = golden("small.vcf");
    let out = run(&["freq", vcf.to_str().unwrap()]);
    // chr1:400 is all ref (0/0 for all 5 samples); alt freq = 0.0, still emitted
    // (vcftools --freq does emit monomorphic sites)
    let lines: Vec<&str> = out.lines().collect();
    let site400 = lines.iter().find(|l| l.contains("chr1\t400\t"));
    assert!(
        site400.is_some(),
        "monomorphic site 400 should still be emitted"
    );
}

// ─── het ─────────────────────────────────────────────────────────────────────

#[test]
fn het_has_header() {
    let vcf = golden("small.vcf");
    let out = run(&["het", vcf.to_str().unwrap()]);
    assert!(out.starts_with("INDV\tO(HOM)\tE(HOM)\tN_SITES\tF"));
}

#[test]
fn het_f_in_range() {
    let vcf = golden("small.vcf");
    let out = run(&["het", vcf.to_str().unwrap()]);
    let lines: Vec<&str> = out.lines().skip(1).collect();
    // 5 samples expected.
    assert_eq!(lines.len(), 5, "5 samples in het output");
    for line in &lines {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(cols.len(), 5, "5 columns in het output: {line}");
        let f: f64 = cols[4].parse().unwrap_or(f64::NAN);
        if !f.is_nan() {
            assert!((-2.0..=2.0).contains(&f), "F out of plausible range: {f}");
        }
    }
}

// ─── hardy ───────────────────────────────────────────────────────────────────

#[test]
fn hardy_has_header() {
    let vcf = golden("small.vcf");
    let out = run(&["hardy", vcf.to_str().unwrap()]);
    assert!(out.starts_with("CHROM\tPOS\tOBS(HOM1/HET/HOM2)"));
}

#[test]
fn hardy_p_in_range() {
    let vcf = golden("small.vcf");
    let out = run(&["hardy", vcf.to_str().unwrap()]);
    for line in out.lines().skip(1) {
        let cols: Vec<&str> = line.split('\t').collect();
        assert!(cols.len() >= 8, "8 cols in hardy output: {line}");
        let p_hwe: f64 = cols[5].parse().unwrap_or(f64::NAN);
        if !p_hwe.is_nan() {
            assert!((0.0..=1.0).contains(&p_hwe), "P_HWE out of [0,1]: {p_hwe}");
        }
    }
}

// ─── missing-site ────────────────────────────────────────────────────────────

#[test]
fn missing_site_has_header() {
    let vcf = golden("small.vcf");
    let out = run(&["missing-site", vcf.to_str().unwrap()]);
    assert!(out.starts_with("CHROM\tPOS\tN_DATA"));
}

#[test]
fn missing_site_counts_missing() {
    let vcf = golden("small.vcf");
    let out = run(&["missing-site", vcf.to_str().unwrap()]);
    // chr1:300 has 1 missing (./.); chr1:500 has 1 missing (./.); others 0.
    let lines: Vec<&str> = out.lines().skip(1).collect();
    // 8 sites total in small.vcf.
    assert_eq!(lines.len(), 8);
    let site300 = lines.iter().find(|l| l.contains("chr1\t300\t")).unwrap();
    let cols: Vec<&str> = site300.split('\t').collect();
    assert_eq!(cols[4], "1", "1 missing at chr1:300");
    assert_eq!(cols[5], "0.2000", "20% missing at chr1:300");
}

// ─── missing-indv ────────────────────────────────────────────────────────────

#[test]
fn missing_indv_has_header() {
    let vcf = golden("small.vcf");
    let out = run(&["missing-indv", vcf.to_str().unwrap()]);
    assert!(out.starts_with("INDV\tN_DATA"));
}

#[test]
fn missing_indv_counts() {
    let vcf = golden("small.vcf");
    let out = run(&["missing-indv", vcf.to_str().unwrap()]);
    let lines: Vec<&str> = out.lines().skip(1).collect();
    assert_eq!(lines.len(), 5, "5 samples");
    // SAMPLE2 has ./. at pos 500 → 1 missing.
    let s2 = lines.iter().find(|l| l.starts_with("SAMPLE2")).unwrap();
    let cols: Vec<&str> = s2.split('\t').collect();
    assert_eq!(cols[3], "1", "SAMPLE2 has 1 missing");
}

// ─── pi ──────────────────────────────────────────────────────────────────────

#[test]
fn pi_has_header() {
    let vcf = golden("small.vcf");
    let out = run(&["pi", vcf.to_str().unwrap(), "--window", "1000000"]);
    assert!(out.starts_with("CHROM\tBIN_START\tBIN_END\tN_VARIANTS\tPI"));
}

#[test]
fn pi_nonnegative() {
    let vcf = golden("small.vcf");
    let out = run(&["pi", vcf.to_str().unwrap(), "--window", "1000000"]);
    for line in out.lines().skip(1) {
        let cols: Vec<&str> = line.split('\t').collect();
        let pi: f64 = cols[4].parse().unwrap_or(f64::NAN);
        assert!(pi >= 0.0, "pi should be non-negative: {pi}");
    }
}

// ─── singleton ───────────────────────────────────────────────────────────────

#[test]
fn singleton_has_header() {
    let vcf = golden("small.vcf");
    let out = run(&["singleton", vcf.to_str().unwrap()]);
    assert!(out.starts_with("CHROM\tPOS\tSINGLETON/DOUBLETON\tALLELE\tINDV"));
}

#[test]
fn singleton_type_is_s_or_d() {
    let vcf = golden("small.vcf");
    let out = run(&["singleton", vcf.to_str().unwrap()]);
    for line in out.lines().skip(1) {
        let cols: Vec<&str> = line.split('\t').collect();
        assert!(
            cols[2] == "S" || cols[2] == "D",
            "type must be S or D: {}",
            cols[2]
        );
    }
}

// ─── CLI ─────────────────────────────────────────────────────────────────────

#[test]
fn cli_help_exits_zero() {
    let out = Command::new(bin())
        .arg("--help")
        .output()
        .expect("run rsomics-vcf-popgen --help");
    assert!(out.status.success(), "help should exit 0");
}
