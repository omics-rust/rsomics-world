//! Compatibility test: compare `rsomics-vcf-setgt` output vs `bcftools +setGT`.
//!
//! Skips automatically when bcftools is absent or the setGT plugin is unavailable.
//! Version-gates to bcftools ≥1.0 (any modern release).

use std::path::{Path, PathBuf};
use std::process::Command;

fn bcftools_version() -> Option<String> {
    let out = Command::new("bcftools").arg("--version").output().ok()?;
    Some(
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .next()?
            .to_owned(),
    )
}

fn setgt_plugin_available() -> bool {
    // Probe with a minimal no-op invocation; return false on any error.
    Command::new("bcftools")
        .args(["+setGT", "--version"])
        .output()
        .is_ok_and(|o| o.status.success() || !o.stderr.is_empty())
}

fn golden() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/small.vcf")
}

fn run_bcftools(input: &Path, output: &Path, target: &str, new_gt: &str) -> bool {
    Command::new("bcftools")
        .args([
            "+setGT",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "--",
            "-t",
            target,
            "-n",
            new_gt,
        ])
        .status()
        .is_ok_and(|s| s.success())
}

fn run_ours(input: &Path, output: &Path, target: &str, new_gt: &str) {
    let bin = env!("CARGO_BIN_EXE_rsomics-vcf-setgt");
    let status = Command::new(bin)
        .args([
            input.to_str().unwrap(),
            "--target",
            target,
            "-n",
            new_gt,
            "-o",
            output.to_str().unwrap(),
        ])
        .status()
        .expect("failed to run rsomics-vcf-setgt");
    assert!(status.success(), "rsomics-vcf-setgt exited non-zero");
}

fn data_lines(vcf: &Path) -> Vec<String> {
    std::fs::read_to_string(vcf)
        .expect("read vcf")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .map(str::to_owned)
        .collect()
}

fn check_case(target: &str, new_gt: &str) {
    let input = golden();
    let tmp = tempfile::tempdir().expect("tmpdir");
    let bc_out = tmp.path().join("bc.vcf");
    let our_out = tmp.path().join("ours.vcf");

    let ok = run_bcftools(&input, &bc_out, target, new_gt);
    assert!(ok, "bcftools +setGT failed for -t {target} -n {new_gt}");
    run_ours(&input, &our_out, target, new_gt);

    let bc_lines = data_lines(&bc_out);
    let our_lines = data_lines(&our_out);
    assert_eq!(
        bc_lines.len(),
        our_lines.len(),
        "-t {target} -n {new_gt}: line count differs"
    );
    for (i, (bc, ours)) in bc_lines.iter().zip(our_lines.iter()).enumerate() {
        assert_eq!(
            bc, ours,
            "-t {target} -n {new_gt}: line {i} differs\n  bcftools: {bc}\n  ours:     {ours}"
        );
    }
}

fn skip_if_unavailable() -> bool {
    if bcftools_version().is_none() {
        eprintln!("SKIP: bcftools not found");
        return true;
    }
    if !setgt_plugin_available() {
        eprintln!("SKIP: bcftools +setGT plugin not available");
        return true;
    }
    false
}

#[test]
fn compat_missing_to_ref() {
    if skip_if_unavailable() {
        return;
    }
    check_case(".", "0");
}

#[test]
fn compat_all_to_missing() {
    if skip_if_unavailable() {
        return;
    }
    check_case("a", ".");
}

#[test]
fn compat_fully_missing_to_ref() {
    if skip_if_unavailable() {
        return;
    }
    check_case("./.", "0");
}

#[test]
fn compat_all_phase() {
    if skip_if_unavailable() {
        return;
    }
    check_case("a", "p");
}

#[test]
fn compat_all_unphase() {
    if skip_if_unavailable() {
        return;
    }
    check_case("a", "u");
}

#[test]
fn compat_all_custom_homref() {
    if skip_if_unavailable() {
        return;
    }
    check_case("a", "c:0/0");
}
