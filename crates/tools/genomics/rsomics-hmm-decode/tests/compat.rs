use std::process::Command;
fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-hmm-decode"))
}
fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn output_length_matches_input() {
    let out = Command::new(ours())
        .args(["-m", &golden("model.json")])
        .arg(golden("obs.txt"))
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    // Input has 2 lines of 5 observations each → 2 output lines of 5 states
    let lines: Vec<&str> = s.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    for line in &lines {
        let states: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(states.len(), 5, "each state sequence should have 5 states");
    }
}

#[test]
fn states_are_valid() {
    // 2-state HMM → states should be 0 or 1
    let out = Command::new(ours())
        .args(["-m", &golden("model.json")])
        .arg(golden("obs.txt"))
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    for state in s.split_whitespace() {
        let s: usize = state.parse().unwrap();
        assert!(
            s <= 1,
            "2-state HMM should only produce states 0 or 1, got {s}"
        );
    }
}
