use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-kmer-dist"))
}

fn golden(name: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), name)
}

#[test]
fn jaccard_two_files() {
    let out = bin()
        .args([&golden("a.fa"), &golden("b.fa"), "-k", "3", "-m", "jaccard"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains('\t'));
    let val: f64 = s.trim().split('\t').nth(2).unwrap().parse().unwrap();
    assert!((0.0..=1.0).contains(&val));
}
