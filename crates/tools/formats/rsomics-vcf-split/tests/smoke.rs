use std::path::Path;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-vcf-split"))
}

fn fixture() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/small.vcf"
    ))
}

#[test]
fn split_by_chrom() {
    let dir = std::env::temp_dir().join("rsomics-vcf-split-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let prefix = dir.join("out");

    let out = bin()
        .arg(fixture())
        .arg("-o")
        .arg(&prefix)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let entries: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert!(entries.len() >= 2);

    let _ = std::fs::remove_dir_all(&dir);
}
