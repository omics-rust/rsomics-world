use std::process::Command;
fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-kraken-report"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn top_species() {
    let out = bin()
        .arg(golden("report.txt"))
        .args(["-r", "S", "-n", "5"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("Escherichia coli"));
    let data: Vec<&str> = s.trim().lines().skip(1).collect();
    assert_eq!(data.len(), 2);
}
