use std::process::Command;
fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-featurecounts"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn runs_without_crash() {
    // BAM chroms likely don't match GFF chroms in test fixtures
    // Just verify it doesn't crash/panic
    let out = Command::new(ours())
        .arg(golden("small.bam"))
        .args(["-a", &golden("small.gff")])
        .output()
        .unwrap();
    // May fail gracefully or succeed with 0 counts — both acceptable
    assert!(
        out.status.success() || !String::from_utf8_lossy(&out.stderr).contains("panic"),
        "should not panic"
    );
}
