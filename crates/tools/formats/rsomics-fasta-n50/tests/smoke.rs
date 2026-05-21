use std::path::Path;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-fasta-n50"))
}

fn fixture() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/small.fa"
    ))
}

#[test]
fn basic_stats() {
    let out = bin().arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("sequences\t5"));
    assert!(s.contains("N50\t"));
    assert!(s.contains("GC%\t"));
}

#[test]
fn json_output() {
    let out = bin().arg("--json").arg(fixture()).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"num_seqs\""));
    assert!(s.contains("\"n50\""));
    assert!(s.contains("\"gc_pct\""));
}
