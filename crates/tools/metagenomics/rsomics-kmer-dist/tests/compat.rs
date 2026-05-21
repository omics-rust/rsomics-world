use std::process::Command;
fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-kmer-dist"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn jaccard_in_0_1() {
    let out = Command::new(ours())
        .arg(golden("a.fa"))
        .arg(golden("b.fa"))
        .args(["-k", "3", "-m", "jaccard"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    let d: f64 = s.trim().split('\t').nth(2).unwrap().parse().unwrap();
    assert!(
        (0.0..=1.0).contains(&d),
        "Jaccard distance must be in [0,1], got {d}"
    );
}

#[test]
fn self_distance_is_zero() {
    let out = Command::new(ours())
        .arg(golden("a.fa"))
        .arg(golden("a.fa"))
        .args(["-k", "3", "-m", "jaccard"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    let d: f64 = s.trim().split('\t').nth(2).unwrap().parse().unwrap();
    assert!((d).abs() < 0.001, "self-distance should be 0, got {d}");
}
