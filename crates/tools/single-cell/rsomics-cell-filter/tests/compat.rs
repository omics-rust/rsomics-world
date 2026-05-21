use std::process::Command;

fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-cell-filter"))
}

fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn strict_filter_removes_all() {
    let out = Command::new(ours())
        .arg(golden("stats.tsv"))
        .args([
            "--min-genes",
            "10000",
            "--min-umis",
            "10000",
            "--max-mito",
            "0.01",
        ])
        .output()
        .unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    let data: Vec<&str> = s.lines().skip(1).collect();
    assert_eq!(data.len(), 0, "strict filter should remove all cells");
}

#[test]
fn relaxed_filter_keeps_all() {
    let out = Command::new(ours())
        .arg(golden("stats.tsv"))
        .args(["--min-genes", "1", "--min-umis", "1", "--max-mito", "1.0"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    let data: Vec<&str> = s.lines().skip(1).collect();
    assert_eq!(data.len(), 3, "relaxed filter should keep all 3 cells");
}

#[test]
fn intermediate_filter() {
    let out = Command::new(ours())
        .arg(golden("stats.tsv"))
        .args([
            "--min-genes",
            "200",
            "--min-umis",
            "500",
            "--max-mito",
            "0.2",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    let data: Vec<&str> = s.lines().skip(1).collect();
    // CELL1 (500 genes, 1000 UMI, 0.05 mito) → passes
    // CELL2 (100 genes, 200 UMI, 0.3 mito) → fails all 3
    // CELL3 (300 genes, 800 UMI, 0.1 mito) → passes
    assert_eq!(data.len(), 2, "should keep CELL1 + CELL3");
}
