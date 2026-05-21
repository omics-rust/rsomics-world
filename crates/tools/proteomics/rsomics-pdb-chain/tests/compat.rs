use std::process::Command;
fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-pdb-chain"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn extract_each_listed_chain() {
    let list_out = Command::new(ours())
        .args(["list", &golden("small.pdb")])
        .output()
        .unwrap();
    assert!(list_out.status.success());
    let list_text = String::from_utf8(list_out.stdout).unwrap();
    let chains: Vec<&str> = list_text
        .trim()
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    for chain in &chains {
        let extract = Command::new(ours())
            .args(["extract", &golden("small.pdb"), "-c", chain])
            .output()
            .unwrap();
        assert!(extract.status.success(), "extract chain {chain} failed");
        let s = String::from_utf8(extract.stdout).unwrap();
        let atoms: Vec<&str> = s.lines().filter(|l| l.starts_with("ATOM")).collect();
        assert!(!atoms.is_empty(), "chain {chain} should have atoms");
    }
}
