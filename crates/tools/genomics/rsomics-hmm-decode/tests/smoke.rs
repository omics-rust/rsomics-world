use std::process::Command;
fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-hmm-decode"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn viterbi_decode() {
    let out = bin()
        .args(["-m", &golden("model.json")])
        .arg(golden("obs.txt"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 2);
}
