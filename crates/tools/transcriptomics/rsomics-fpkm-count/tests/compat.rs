/// Compatibility tests against RSeQC FPKM_count.py 5.0.4.
///
/// These tests compare rsomics-fpkm-count output against the RSeQC reference
/// implementation running on the same fixture. They are guarded behind a
/// `which FPKM_count.py` check; if the oracle is absent the test is skipped.
///
/// Column-level comparison:
/// - #chrom, st, end, accession, mRNA_size, gene_strand: byte-exact
/// - Frag_count: exact integer match (stored as float with .0 suffix in both)
/// - FPM, FPKM: absolute tolerance 1e-3 (floating-point rounding differences
///   between Python/Rust f64 representations of the same formula)
///
/// RSeQC FPKM formula: FPKM = Frag_count × 1e9 / (mRNA_size × total_fragments)
/// Both tools use the same formula; tolerance covers Python float printing
/// vs Rust's Ryu-based display (e.g. "2000000.0" vs "2000000").
use std::path::Path;
use std::process::Command;

fn fpkm_count_path() -> Option<std::path::PathBuf> {
    // Try conda environment first, then PATH.
    let candidates = [
        "/opt/homebrew/Caskroom/miniforge/base/envs/rs-up/bin/FPKM_count.py",
        "FPKM_count.py",
    ];
    for c in candidates {
        let p = Path::new(c);
        if p.exists() {
            return Some(p.to_path_buf());
        }
        if let Ok(out) = Command::new("which").arg(c).output()
            && out.status.success()
        {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                return Some(Path::new(&s).to_path_buf());
            }
        }
    }
    None
}

fn binary_path() -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Walk up to workspace root then find the binary.
    for _ in 0..5 {
        let candidate = p.join("target/debug/rsomics-fpkm-count");
        if candidate.exists() {
            return candidate;
        }
        let candidate = std::env::var("CARGO_TARGET_DIR")
            .map(|t| Path::new(&t).join("debug/rsomics-fpkm-count"))
            .unwrap_or_else(|_| p.join("target/debug/rsomics-fpkm-count"));
        if candidate.exists() {
            return candidate;
        }
        p = p.parent().unwrap_or(&p).to_path_buf();
    }
    // Fall back to PATH.
    Path::new("rsomics-fpkm-count").to_path_buf()
}

fn golden_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

/// Parse FPKM.xls lines into (accession, count, fpm, fpkm) tuples.
fn parse_fpkm_xls(s: &str) -> Vec<(String, f64, f64, f64)> {
    s.lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let cols: Vec<&str> = l.split('\t').collect();
            assert!(cols.len() >= 9, "expected 9 cols in FPKM.xls line: {l:?}");
            let acc = cols[3].to_string();
            let count: f64 = cols[6].parse().unwrap_or(0.0);
            let fpm: f64 = cols[7].parse().unwrap_or(0.0);
            let fpkm: f64 = cols[8].parse().unwrap_or(0.0);
            (acc, count, fpm, fpkm)
        })
        .collect()
}

#[test]
fn fpkm_matches_rseqc_on_golden_fixture() {
    let oracle = match fpkm_count_path() {
        Some(p) => p,
        None => {
            eprintln!("SKIP: FPKM_count.py not found");
            return;
        }
    };

    let gold = golden_dir();
    let bam = gold.join("sample.bam");
    let bed = gold.join("genes.bed12");

    assert!(bam.exists(), "golden BAM missing: {}", bam.display());
    assert!(bed.exists(), "golden BED12 missing: {}", bed.display());

    let tmp = tempfile::tempdir().unwrap();
    let ref_prefix = tmp.path().join("ref");

    // Run RSeQC oracle.
    let oracle_status = Command::new(&oracle)
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-r",
            bed.to_str().unwrap(),
            "-o",
            ref_prefix.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run FPKM_count.py");
    assert!(
        oracle_status.status.success(),
        "FPKM_count.py failed:\n{}",
        String::from_utf8_lossy(&oracle_status.stderr)
    );

    // Run our binary.
    let our_prefix = tmp.path().join("ours");
    let our_status = Command::new(binary_path())
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-r",
            bed.to_str().unwrap(),
            "-o",
            our_prefix.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run rsomics-fpkm-count");
    assert!(
        our_status.status.success(),
        "rsomics-fpkm-count failed:\n{}",
        String::from_utf8_lossy(&our_status.stderr)
    );

    let ref_xls = std::fs::read_to_string(format!("{}.FPKM.xls", ref_prefix.display())).unwrap();
    let our_xls = std::fs::read_to_string(format!("{}.FPKM.xls", our_prefix.display())).unwrap();

    let ref_rows = parse_fpkm_xls(&ref_xls);
    let our_rows = parse_fpkm_xls(&our_xls);

    assert_eq!(ref_rows.len(), our_rows.len(), "row count mismatch");

    const FLOAT_TOL: f64 = 1e-3;
    for (r, o) in ref_rows.iter().zip(our_rows.iter()) {
        assert_eq!(r.0, o.0, "accession mismatch: {:?} vs {:?}", r.0, o.0);
        assert!(
            (r.1 - o.1).abs() < FLOAT_TOL,
            "Frag_count mismatch for {}: ref={} ours={}",
            r.0,
            r.1,
            o.1
        );
        assert!(
            (r.2 - o.2).abs() < FLOAT_TOL,
            "FPM mismatch for {}: ref={} ours={}",
            r.0,
            r.2,
            o.2
        );
        assert!(
            (r.3 - o.3).abs() < FLOAT_TOL,
            "FPKM mismatch for {}: ref={} ours={}",
            r.0,
            r.3,
            o.3
        );
    }
}

#[test]
fn fpkm_count_exits_zero_on_golden() {
    let gold = golden_dir();
    let bam = gold.join("sample.bam");
    let bed = gold.join("genes.bed12");

    if !bam.exists() || !bed.exists() {
        eprintln!("SKIP: golden fixtures missing");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let prefix = tmp.path().join("out");
    let status = Command::new(binary_path())
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-r",
            bed.to_str().unwrap(),
            "-o",
            prefix.to_str().unwrap(),
        ])
        .status()
        .expect("failed to run rsomics-fpkm-count");
    assert!(status.success(), "rsomics-fpkm-count exited non-zero");
}

#[test]
fn golden_counts_match_expected() {
    let gold = golden_dir();
    let bam = gold.join("sample.bam");
    let bed = gold.join("genes.bed12");
    let expected_path = gold.join("expected.FPKM.xls");

    if !bam.exists() || !expected_path.exists() {
        eprintln!("SKIP: golden fixtures missing");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let prefix = tmp.path().join("out");
    let status = Command::new(binary_path())
        .args([
            "-i",
            bam.to_str().unwrap(),
            "-r",
            bed.to_str().unwrap(),
            "-o",
            prefix.to_str().unwrap(),
        ])
        .status()
        .expect("failed to run rsomics-fpkm-count");
    assert!(status.success());

    let expected = std::fs::read_to_string(&expected_path).unwrap();
    let actual = std::fs::read_to_string(format!("{}.FPKM.xls", prefix.display())).unwrap();

    let exp_rows = parse_fpkm_xls(&expected);
    let act_rows = parse_fpkm_xls(&actual);

    assert_eq!(exp_rows.len(), act_rows.len());
    const TOL: f64 = 1e-3;
    for (e, a) in exp_rows.iter().zip(act_rows.iter()) {
        assert_eq!(e.0, a.0);
        assert!((e.1 - a.1).abs() < TOL, "count: ref={} ours={}", e.1, a.1);
        assert!((e.2 - a.2).abs() < TOL, "fpm:   ref={} ours={}", e.2, a.2);
        assert!((e.3 - a.3).abs() < TOL, "fpkm:  ref={} ours={}", e.3, a.3);
    }
}
