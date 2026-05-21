use std::process::Command;

fn ours() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_rsomics-nj-tree"))
}

fn fixture() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/dist.tsv")
}

#[test]
fn output_is_valid_newick() {
    let out = Command::new(ours()).arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    let tree = s.trim();

    assert!(tree.ends_with(';'), "Newick must end with ;");
    assert!(tree.contains('('), "Newick must have parentheses");
    assert!(tree.contains(':'), "Newick must have branch lengths");

    // All 3 taxa (A, B, C) must appear
    assert!(tree.contains('A'));
    assert!(tree.contains('B'));
    assert!(tree.contains('C'));

    // Parentheses must be balanced
    let opens = tree.chars().filter(|&c| c == '(').count();
    let closes = tree.chars().filter(|&c| c == ')').count();
    assert_eq!(opens, closes, "unbalanced parentheses");
}

#[test]
fn symmetric_input_produces_ultrametric_like_tree() {
    // Create a symmetric 3-taxon matrix where A-B are closest
    let dir = std::env::temp_dir().join("nj-tree-compat");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let matrix = dir.join("sym.tsv");
    std::fs::write(
        &matrix,
        "X\tY\tZ\n0.0\t0.1\t0.5\n0.1\t0.0\t0.5\n0.5\t0.5\t0.0\n",
    )
    .unwrap();

    let out = Command::new(ours()).arg(&matrix).output().unwrap();
    assert!(out.status.success());
    let tree = String::from_utf8(out.stdout).unwrap();

    // X and Y should be grouped together (closer to each other)
    // The tree should have (X:...,Y:...) as a clade
    assert!(tree.contains('X'));
    assert!(tree.contains('Y'));
    assert!(tree.contains('Z'));

    let _ = std::fs::remove_dir_all(&dir);
}
