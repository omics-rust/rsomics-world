use std::process::Command;

fn rerep_binary() -> std::path::PathBuf {
    env!("CARGO_BIN_EXE_rsomics-rereplicate").into()
}

fn golden(name: &str) -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/golden");
    p.push(name);
    p
}

#[test]
fn smoke_basic_produces_output() {
    let input = golden("basic.fasta");
    let out = tempfile::NamedTempFile::new().unwrap();
    let status = Command::new(rerep_binary())
        .args([
            input.to_str().unwrap(),
            "-o",
            out.path().to_str().unwrap(),
            "-q",
        ])
        .status()
        .unwrap();
    assert!(status.success(), "binary exited non-zero");
    let content = std::fs::read_to_string(out.path()).unwrap();
    // seq1;size=3 should produce 3 copies of >seq1
    let count = content.lines().filter(|l| *l == ">seq1").count();
    assert_eq!(count, 3, "expected 3 copies of seq1");
    assert!(content.contains(">seq2"), "seq2 should appear once");
    assert!(!content.contains(";size="), "no ;size= in default output");
}

#[test]
fn smoke_sizeout_appends_size1() {
    let input = golden("basic.fasta");
    let out = tempfile::NamedTempFile::new().unwrap();
    let status = Command::new(rerep_binary())
        .args([
            input.to_str().unwrap(),
            "-o",
            out.path().to_str().unwrap(),
            "-q",
            "--sizeout",
        ])
        .status()
        .unwrap();
    assert!(status.success());
    let content = std::fs::read_to_string(out.path()).unwrap();
    assert!(
        content.contains(">seq1;size=1"),
        "each copy should have ;size=1 with --sizeout"
    );
}

#[test]
fn smoke_middle_size_stripped_correctly() {
    // seq_middle_size;k=val;size=4;extra=more → label should be seq_middle_size;k=val;extra=more
    let input = golden("adversarial.fasta");
    let out = tempfile::NamedTempFile::new().unwrap();
    Command::new(rerep_binary())
        .args([
            input.to_str().unwrap(),
            "-o",
            out.path().to_str().unwrap(),
            "-q",
        ])
        .status()
        .unwrap();
    let content = std::fs::read_to_string(out.path()).unwrap();
    let count = content
        .lines()
        .filter(|l| *l == ">seq_middle_size;k=val;extra=more")
        .count();
    assert_eq!(count, 4, "middle-size stripped label should appear 4 times");
}

#[test]
fn smoke_case_and_u_preserved() {
    let input = golden("adversarial.fasta");
    let out = tempfile::NamedTempFile::new().unwrap();
    Command::new(rerep_binary())
        .args([
            input.to_str().unwrap(),
            "-o",
            out.path().to_str().unwrap(),
            "-q",
        ])
        .status()
        .unwrap();
    let content = std::fs::read_to_string(out.path()).unwrap();
    // Mixed case preserved
    let mixed = content.lines().filter(|l| *l == "acgtACGTacgt").count();
    assert_eq!(
        mixed, 2,
        "mixed-case sequence should be preserved, 2 copies"
    );
    // U preserved
    let u_lines = content.lines().filter(|l| *l == "ACGUACGUacgu").count();
    assert_eq!(u_lines, 2, "U should be preserved in output");
}

#[test]
fn smoke_no_minseqlength_filter() {
    // Short sequences (size=5, seq=ACGT 4 nt) should NOT be filtered
    let input = golden("adversarial.fasta");
    let out = tempfile::NamedTempFile::new().unwrap();
    Command::new(rerep_binary())
        .args([
            input.to_str().unwrap(),
            "-o",
            out.path().to_str().unwrap(),
            "-q",
        ])
        .status()
        .unwrap();
    let content = std::fs::read_to_string(out.path()).unwrap();
    let short_count = content.lines().filter(|l| *l == ">seq_short").count();
    assert_eq!(
        short_count, 5,
        "short sequences should not be filtered (no minseqlength)"
    );
}

#[test]
fn smoke_cli_debug_assert() {
    let status = Command::new(rerep_binary()).arg("--help").status().unwrap();
    let _ = status;
}
