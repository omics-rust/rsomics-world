use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-fastq-pair"))
}

fn golden(name: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), name)
}

#[test]
fn pair_matching_reads() {
    let dir = std::env::temp_dir().join("rsomics-fastq-pair-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let out1 = dir.join("paired_r1.fq");
    let out2 = dir.join("paired_r2.fq");

    let out = bin()
        .arg(golden("r1.fq"))
        .arg(golden("r2.fq"))
        .arg("--out1")
        .arg(&out1)
        .arg("--out2")
        .arg(&out2)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let r1 = std::fs::read_to_string(&out1).unwrap();
    let r2 = std::fs::read_to_string(&out2).unwrap();
    let r1_names: Vec<&str> = r1.lines().filter(|l| l.starts_with('@')).collect();
    let r2_names: Vec<&str> = r2.lines().filter(|l| l.starts_with('@')).collect();
    assert_eq!(r1_names.len(), 3);
    assert_eq!(r2_names.len(), 3);

    let _ = std::fs::remove_dir_all(&dir);
}
