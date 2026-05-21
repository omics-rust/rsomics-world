use std::process::Command;
fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-cell-filter"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn filter_cells() {
    let out = bin()
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
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    let data: Vec<&str> = s.trim().lines().skip(1).collect();
    assert_eq!(data.len(), 2); // CELL1 and CELL3 pass, CELL2 filtered
}
