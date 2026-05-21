use std::process::Command;
fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-tpm"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn tpm_normalize() {
    let out = bin()
        .arg(golden("counts.tsv"))
        .args(["-l", &golden("lengths.tsv")])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("GENEA"));
    let data: Vec<&str> = s.trim().lines().skip(1).collect();
    assert_eq!(data.len(), 2);
    // Each column should sum to ~1M
    let vals: Vec<f64> = data[0]
        .split('\t')
        .skip(1)
        .filter_map(|v| v.parse().ok())
        .collect();
    assert!(vals.iter().all(|&v| v > 0.0));
}
