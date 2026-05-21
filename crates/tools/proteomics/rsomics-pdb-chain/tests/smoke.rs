use std::process::Command;
fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-pdb-chain"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn list_chains() {
    let out = bin().args(["list", &golden("small.pdb")]).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    let chains: Vec<&str> = s.trim().lines().collect();
    assert_eq!(chains, vec!["A", "B"]);
}

#[test]
fn extract_chain_a() {
    let out = bin()
        .args(["extract", &golden("small.pdb"), "-c", "A"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let atoms: Vec<&str> = s.lines().filter(|l| l.starts_with("ATOM")).collect();
    assert_eq!(atoms.len(), 2);
}
