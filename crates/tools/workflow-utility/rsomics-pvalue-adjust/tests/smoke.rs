use std::io::Write;
use std::process::{Command, Stdio};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-pvalue-adjust"))
}

#[test]
fn bh_adjust() {
    let mut child = bin()
        .args(["--method", "BH"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let stdin = child.stdin.as_mut().unwrap();
    stdin.write_all(b"0.01\n0.04\n0.03\n0.20\n0.50\n").unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let s = String::from_utf8_lossy(&output.stdout);
    let vals: Vec<f64> = s
        .trim()
        .lines()
        .filter_map(|l| l.split('\t').nth(1)?.parse().ok())
        .collect();
    assert_eq!(vals.len(), 5);
    assert!(vals[0] >= 0.01);
}
