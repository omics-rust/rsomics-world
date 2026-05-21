use std::process::Command;

fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-align-score"))
}

#[test]
fn identical_sequences_identity_1() {
    let dir = std::env::temp_dir().join("align-compat-ident");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let fa = dir.join("ident.fa");
    std::fs::write(&fa, ">a\nATCGATCG\n>b\nATCGATCG\n").unwrap();

    let out = Command::new(ours()).arg(&fa).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    let line = s.lines().nth(1).unwrap();
    let score: i32 = line.split('\t').nth(2).unwrap().parse().unwrap();
    let identity: f64 = line.split('\t').nth(3).unwrap().parse().unwrap();

    assert_eq!(score, 8, "8bp identical with match=1 should score 8");
    assert!(
        (identity - 1.0).abs() < 0.001,
        "identical seqs should have identity=1.0, got {identity}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn score_increases_with_similarity() {
    let dir = std::env::temp_dir().join("align-compat-sim");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let similar = dir.join("sim.fa");
    std::fs::write(&similar, ">a\nATCGATCG\n>b\nATCGATCG\n").unwrap();
    let different = dir.join("diff.fa");
    std::fs::write(&different, ">a\nATCGATCG\n>b\nTTTTTTTT\n").unwrap();

    let out_sim = Command::new(ours()).arg(&similar).output().unwrap();
    let out_diff = Command::new(ours()).arg(&different).output().unwrap();

    let score_sim: i32 = String::from_utf8(out_sim.stdout)
        .unwrap()
        .lines()
        .nth(1)
        .unwrap()
        .split('\t')
        .nth(2)
        .unwrap()
        .parse()
        .unwrap();
    let score_diff: i32 = String::from_utf8(out_diff.stdout)
        .unwrap()
        .lines()
        .nth(1)
        .unwrap()
        .split('\t')
        .nth(2)
        .unwrap()
        .parse()
        .unwrap();

    assert!(
        score_sim > score_diff,
        "identical should score higher ({score_sim}) than different ({score_diff})"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
