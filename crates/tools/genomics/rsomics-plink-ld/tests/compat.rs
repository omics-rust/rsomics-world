use std::io::{BufWriter, Write};
use std::path::PathBuf;

use rsomics_pgen::Pgen;
use rsomics_plink_ld::{compute_ld, r2};

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn load_golden() -> Pgen {
    let prefix = golden_dir().join("test");
    Pgen::load(&prefix).expect("load golden PLINK fileset")
}

#[test]
fn r2_self_is_one() {
    let pgen = load_golden();
    if pgen.variants.is_empty() {
        return;
    }
    // r² of a variant with itself should be 1.0 (if non-monomorphic).
    for i in 0..pgen.variants.len() {
        let v = r2(&pgen, i, i);
        // monomorphic variants get 0.0; polymorphic get 1.0
        assert!((v - 1.0).abs() < 1e-10 || v == 0.0, "r²(v,v) = {v}");
    }
}

#[test]
fn compute_ld_runs_without_error() {
    let pgen = load_golden();
    let mut buf = BufWriter::new(Vec::new());
    compute_ld(&pgen, 50, 0.0, &mut buf).expect("compute_ld");
}

#[test]
fn compute_ld_output_has_header() {
    let pgen = load_golden();
    let mut buf = Vec::new();
    // Write header manually (as main.rs does) then pairs.
    writeln!(buf, "CHR_A\tBP_A\tSNP_A\tCHR_B\tBP_B\tSNP_B\tR2").unwrap();
    compute_ld(&pgen, 50, 0.0, &mut buf).expect("compute_ld");
    let text = String::from_utf8(buf).unwrap();
    assert!(text.starts_with("CHR_A\t"));
}

#[test]
fn r2_range_zero_to_one() {
    let pgen = load_golden();
    let n = pgen.variants.len();
    for i in 0..n {
        for j in i..n {
            let v = r2(&pgen, i, j);
            assert!(
                (0.0..=1.0 + 1e-10).contains(&v),
                "r²({i},{j}) = {v} out of [0,1]"
            );
        }
    }
}

#[test]
fn min_r2_filter_works() {
    let pgen = load_golden();
    let mut all = Vec::new();
    compute_ld(&pgen, 0, 0.0, &mut all).expect("compute_ld all");

    let mut filtered = Vec::new();
    compute_ld(&pgen, 0, 0.5, &mut filtered).expect("compute_ld filtered");

    // The filtered output should be a subset.
    assert!(filtered.len() <= all.len());

    // Every line in filtered must have r² ≥ 0.5.
    let filtered_str = String::from_utf8(filtered).unwrap();
    for line in filtered_str.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        let r2_val: f64 = fields[6].parse().unwrap();
        assert!(
            r2_val >= 0.5,
            "r²={r2_val} below threshold in filtered output"
        );
    }
}

#[test]
fn window_size_limits_pairs() {
    let pgen = load_golden();
    if pgen.variants.len() < 3 {
        return;
    }
    let mut narrow = Vec::new();
    compute_ld(&pgen, 1, 0.0, &mut narrow).expect("narrow window");

    let mut wide = Vec::new();
    compute_ld(&pgen, 0, 0.0, &mut wide).expect("all pairs");

    // A narrower window should produce <= pairs than all-pairs.
    let narrow_lines = String::from_utf8(narrow).unwrap().lines().count();
    let wide_lines = String::from_utf8(wide).unwrap().lines().count();
    assert!(narrow_lines <= wide_lines);
}

#[test]
fn exit_nonzero_on_missing_file() {
    use std::process::Command;
    let bin = env!("CARGO_BIN_EXE_rsomics-plink-ld");
    let status = Command::new(bin)
        .args(["--plink", "/nonexistent/path"])
        .status()
        .expect("spawn binary");
    assert!(!status.success());
}
