use std::path::PathBuf;

use rsomics_pgen::Pgen;
use rsomics_plink_pca::run_pca;

fn load_golden() -> Pgen {
    let prefix = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/test");
    Pgen::load(&prefix).expect("load golden PLINK fileset")
}

#[test]
fn pca_runs_without_error() {
    let pgen = load_golden();
    let mut evec = Vec::new();
    let mut eval = Vec::new();
    run_pca(&pgen, 5, &mut evec, &mut eval).unwrap();
}

#[test]
fn pca_eigenval_count() {
    let pgen = load_golden();
    let mut evec = Vec::new();
    let mut eval = Vec::new();
    run_pca(&pgen, 5, &mut evec, &mut eval).unwrap();
    let eval_str = String::from_utf8(eval).unwrap();
    let n_vals = eval_str.lines().count();
    // Should have min(n_pcs, n_samples - 1) eigenvalues.
    assert!(
        n_vals > 0 && n_vals <= 5,
        "expected 1-5 eigenvalues, got {n_vals}"
    );
}

#[test]
fn pca_eigenvec_has_header() {
    let pgen = load_golden();
    let mut evec = Vec::new();
    let mut eval = Vec::new();
    run_pca(&pgen, 3, &mut evec, &mut eval).unwrap();
    let s = String::from_utf8(evec).unwrap();
    let first = s.lines().next().unwrap();
    assert!(
        first.starts_with("#FID\tIID\tPC1"),
        "unexpected header: {first:?}"
    );
}

#[test]
fn pca_eigenvec_row_count() {
    let pgen = load_golden();
    let n = pgen.n_samples();
    let mut evec = Vec::new();
    let mut eval = Vec::new();
    run_pca(&pgen, 3, &mut evec, &mut eval).unwrap();
    let s = String::from_utf8(evec).unwrap();
    // Header + one row per sample.
    assert_eq!(
        s.lines().count(),
        n + 1,
        "expected {n} data rows + 1 header"
    );
}

#[test]
fn eigenvalues_descending() {
    let pgen = load_golden();
    let mut evec = Vec::new();
    let mut eval = Vec::new();
    run_pca(&pgen, 5, &mut evec, &mut eval).unwrap();
    let vals: Vec<f64> = String::from_utf8(eval)
        .unwrap()
        .lines()
        .map(|l| l.trim().parse::<f64>().unwrap())
        .collect();
    for w in vals.windows(2) {
        assert!(w[0] >= w[1] - 1e-10, "eigenvalues not descending: {w:?}");
    }
}

#[test]
fn pca_scores_finite() {
    let pgen = load_golden();
    let mut evec = Vec::new();
    let mut eval = Vec::new();
    run_pca(&pgen, 3, &mut evec, &mut eval).unwrap();
    let s = String::from_utf8(evec).unwrap();
    for line in s.lines().skip(1) {
        let fields: Vec<&str> = line.split('\t').collect();
        for score in &fields[2..] {
            let v: f64 = score.parse().unwrap();
            assert!(v.is_finite(), "non-finite PC score: {score}");
        }
    }
}

#[test]
fn exit_nonzero_on_missing_file() {
    use std::process::Command;
    let bin = env!("CARGO_BIN_EXE_rsomics-plink-pca");
    let status = Command::new(bin)
        .args(["--plink", "/nonexistent/path"])
        .status()
        .expect("spawn binary");
    assert!(!status.success());
}
