/// Compatibility tests against RSeQC mismatch_profile.py.
///
/// Gated on `RSEQC_MISMATCH_PROFILE` env var pointing to mismatch_profile.py.
/// When the var is absent these tests skip cleanly; on CI / 4090 it is set.
use std::process::Command;

fn rseqc_bin() -> Option<String> {
    std::env::var("RSEQC_MISMATCH_PROFILE").ok()
}

fn our_bin() -> std::path::PathBuf {
    let target_dir = std::env::var("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            // CARGO_MANIFEST_DIR is crates/tools/transcriptomics/rsomics-mismatch-profile;
            // walk up 4 levels to reach the workspace root, then add "target".
            let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            for _ in 0..4 {
                p.pop();
            }
            p.join("target")
        });
    target_dir.join("debug").join("rsomics-mismatch-profile")
}

fn golden(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join(name)
}

#[test]
fn xls_matches_rseqc_small() {
    let rseqc = match rseqc_bin() {
        Some(p) => p,
        None => {
            eprintln!("SKIP: RSEQC_MISMATCH_PROFILE not set");
            return;
        }
    };

    let bam = golden("small.bam");
    let expected_xls = golden("small.mismatch_profile.xls");
    let tmp = tempfile::tempdir().unwrap();

    // Run RSeQC to produce fresh reference output.
    let rseqc_prefix = tmp.path().join("ref");
    let rseqc_status = Command::new("python3")
        .arg(&rseqc)
        .arg("-i")
        .arg(&bam)
        .arg("-l")
        .arg("50")
        .arg("-n")
        .arg("1000000")
        .arg("-q")
        .arg("0")
        .arg("-o")
        .arg(&rseqc_prefix)
        .status()
        .expect("failed to spawn mismatch_profile.py");
    assert!(
        rseqc_status.success(),
        "mismatch_profile.py exited with: {rseqc_status}"
    );

    let ref_xls = format!("{}.mismatch_profile.xls", rseqc_prefix.display());
    let ref_bytes = std::fs::read(&ref_xls)
        .unwrap_or_else(|e| panic!("cannot read RSeQC output {ref_xls}: {e}"));

    // Verify our golden file still matches RSeQC (detects upstream format drift).
    let golden_bytes = std::fs::read(&expected_xls).unwrap();
    assert_eq!(
        ref_bytes, golden_bytes,
        "RSeQC output drifted from stored golden — regenerate tests/golden/small.mismatch_profile.xls"
    );

    // Run our binary.
    let our_prefix = tmp.path().join("ours");
    let bin = our_bin();
    assert!(
        bin.exists(),
        "our binary not found at {}: run `cargo build -p rsomics-mismatch-profile`",
        bin.display()
    );

    let ours_status = Command::new(&bin)
        .arg("-i")
        .arg(&bam)
        .arg("-l")
        .arg("50")
        .arg("-n")
        .arg("1000000")
        .arg("--mapq")
        .arg("0")
        .arg("-o")
        .arg(&our_prefix)
        .status()
        .expect("failed to spawn rsomics-mismatch-profile");
    assert!(
        ours_status.success(),
        "rsomics-mismatch-profile exited with: {ours_status}"
    );

    let our_xls = format!("{}.mismatch_profile.xls", our_prefix.display());
    let our_bytes =
        std::fs::read(&our_xls).unwrap_or_else(|e| panic!("cannot read our output {our_xls}: {e}"));

    assert_eq!(
        ref_bytes,
        our_bytes,
        "XLS output differs from RSeQC reference.\nExpected:\n{}\nGot:\n{}",
        String::from_utf8_lossy(&ref_bytes),
        String::from_utf8_lossy(&our_bytes),
    );
}
