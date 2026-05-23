/// Compatibility test: compare `rsomics-vcf-fixref` output vs `bcftools +fixref` byte-for-byte.
///
/// The only lines legitimately excluded from comparison are `##bcftools_pluginVersion=…` and
/// `##bcftools_pluginCommand=…`: they embed the bcftools version string, the exact command
/// invocation, and a wall-clock Date — values our tool cannot and should not reproduce.
/// Every other line — including all data records with their `FIXREF=<action>` INFO annotations
/// and the `##INFO=<ID=FIXREF,…>` header — must match verbatim.
///
/// Skips automatically when bcftools is absent or the +fixref plugin is unavailable.
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

fn fixref_plugin_available() -> bool {
    let out = Command::new("bcftools")
        .args(["+fixref", "--help"])
        .output();
    match out {
        Ok(o) => o.status.success() || !o.stderr.is_empty(),
        Err(_) => false,
    }
}

fn run_bcftools_fixref(vcf: &Path, ref_fa: &Path, mode: &str, output: &Path) -> bool {
    Command::new("bcftools")
        .args([
            "+fixref",
            vcf.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "--",
            "-f",
            ref_fa.to_str().unwrap(),
            "-m",
            mode,
        ])
        .status()
        .is_ok_and(|s| s.success())
}

fn run_ours(vcf: &Path, ref_fa: &Path, mode: &str, output: &Path) {
    let bin = env!("CARGO_BIN_EXE_rsomics-vcf-fixref");
    let status = Command::new(bin)
        .args([
            vcf.to_str().unwrap(),
            "-f",
            ref_fa.to_str().unwrap(),
            "-m",
            mode,
            "-o",
            output.to_str().unwrap(),
        ])
        .status()
        .expect("failed to run rsomics-vcf-fixref");
    assert!(status.success(), "rsomics-vcf-fixref exited non-zero");
}

/// Return all VCF lines from a file, excluding the two bcftools-internal provenance
/// headers that embed a version string + command + wall-clock Date.
fn meaningful_lines(vcf: &Path) -> Vec<String> {
    std::fs::read_to_string(vcf)
        .expect("read vcf")
        .lines()
        .filter(|l| {
            !l.starts_with("##bcftools_pluginVersion=")
                && !l.starts_with("##bcftools_pluginCommand=")
        })
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect()
}

fn golden_vcf() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/small.vcf")
}

fn golden_ref() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/ref.fa")
}

fn run_mode_compat(mode: &str) {
    let Some(ver) = bcftools_version() else {
        eprintln!("SKIP fixref_{mode}_matches_bcftools: bcftools not found");
        return;
    };
    if !ver.contains("1.23") {
        eprintln!("SKIP fixref_{mode}_matches_bcftools: need bcftools 1.23.x, got {ver}");
        return;
    }
    if !fixref_plugin_available() {
        eprintln!("SKIP fixref_{mode}_matches_bcftools: bcftools +fixref plugin unavailable");
        return;
    }

    let tmp = tempfile::tempdir().expect("tempdir");
    let ours_out = tmp.path().join(format!("ours_{mode}.vcf"));
    let bcf_out = tmp.path().join(format!("bcftools_{mode}.vcf"));

    run_ours(&golden_vcf(), &golden_ref(), mode, &ours_out);
    if !run_bcftools_fixref(&golden_vcf(), &golden_ref(), mode, &bcf_out) {
        eprintln!("SKIP: bcftools +fixref {mode} failed on golden fixture");
        return;
    }

    eprintln!("bcftools version: {ver}");

    let ours_lines = meaningful_lines(&ours_out);
    let bcf_lines = meaningful_lines(&bcf_out);

    assert_eq!(
        ours_lines.len(),
        bcf_lines.len(),
        "line count mismatch in {mode} mode: ours={} bcftools={}",
        ours_lines.len(),
        bcf_lines.len()
    );

    let diffs: Vec<String> = ours_lines
        .iter()
        .zip(bcf_lines.iter())
        .enumerate()
        .filter_map(|(i, (o, b))| {
            if o == b {
                None
            } else {
                Some(format!(
                    "line {i} differs:\n  ours:     {o:?}\n  bcftools: {b:?}"
                ))
            }
        })
        .collect();

    assert!(
        diffs.is_empty(),
        "{mode} mode output differs from bcftools:\n{}",
        diffs.join("\n")
    );
    eprintln!("compat {mode} OK against {ver}");
}

#[test]
fn fixref_flip_matches_bcftools() {
    run_mode_compat("flip");
}

#[test]
fn fixref_flip_all_matches_bcftools() {
    run_mode_compat("flip-all");
}
