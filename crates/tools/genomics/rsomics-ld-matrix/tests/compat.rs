use std::process::Command;

fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-ld-matrix"))
}

fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn identical_vectors_r2_is_1() {
    let out = Command::new(ours())
        .arg(golden("geno.tsv"))
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    // rs1 and rs2 are identical → r²=1.0
    let r2_line = s
        .lines()
        .find(|l| l.contains("rs1") && l.contains("rs2"))
        .unwrap();
    let r2: f64 = r2_line.split('\t').nth(2).unwrap().parse().unwrap();
    assert!(
        (r2 - 1.0).abs() < 0.0001,
        "identical vectors should have r²=1.0, got {r2}"
    );
}

#[test]
fn r2_is_between_0_and_1() {
    let out = Command::new(ours())
        .arg(golden("geno.tsv"))
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    for line in s.lines().skip(1) {
        let r2: f64 = line.split('\t').nth(2).unwrap().parse().unwrap();
        assert!((0.0..=1.0).contains(&r2), "r² must be in [0,1], got {r2}");
    }
}

#[test]
fn n_pairs_is_correct() {
    let out = Command::new(ours())
        .arg(golden("geno.tsv"))
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    let data: Vec<&str> = s.lines().skip(1).collect();
    // 3 variants → 3 choose 2 = 3 pairs
    assert_eq!(data.len(), 3, "3 variants should produce 3 pairs");
}
