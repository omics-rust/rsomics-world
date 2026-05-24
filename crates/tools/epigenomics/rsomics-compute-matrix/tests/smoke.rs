use std::io::Read;
use std::process::Command;

use flate2::read::GzDecoder;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-compute-matrix"))
}

fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

fn read_gz(path: &str) -> String {
    let f = std::fs::File::open(path).unwrap();
    let mut s = String::new();
    GzDecoder::new(f).read_to_string(&mut s).unwrap();
    s
}

#[test]
fn reference_point_runs_and_shapes() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let out = bin()
        .args(["reference-point", "-S", &golden("signal.bw")])
        .args(["-R", &golden("regions.bed")])
        .args(["-o", tmp.path().to_str().unwrap()])
        .args(["--reference-point", "TSS", "-b", "1000", "-a", "1000"])
        .args(["--bin-size", "50", "-q"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = read_gz(tmp.path().to_str().unwrap());
    let mut lines = text.lines();
    let header = lines.next().unwrap();
    assert!(header.starts_with('@'), "header must start with @");
    let rows: Vec<&str> = lines.collect();
    assert_eq!(rows.len(), 20, "20 regions in");
    for row in &rows {
        let cols: Vec<&str> = row.split('\t').collect();
        // 6 BED cols + (1000+1000)/50 = 40 value cols.
        assert_eq!(cols.len(), 46, "row: {row}");
    }
}

#[test]
fn scale_regions_runs() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let out = bin()
        .args(["scale-regions", "-S", &golden("signal.bw")])
        .args(["-R", &golden("regions.bed")])
        .args(["-o", tmp.path().to_str().unwrap()])
        .args([
            "-m",
            "1000",
            "-b",
            "500",
            "-a",
            "500",
            "--bin-size",
            "50",
            "-q",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let text = read_gz(tmp.path().to_str().unwrap());
    // 6 + (500+1000+500)/50 = 40 value cols.
    let cols = text.lines().nth(1).unwrap().split('\t').count();
    assert_eq!(cols, 46);
}

#[test]
fn body_not_multiple_of_bin_fails() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let out = bin()
        .args(["scale-regions", "-S", &golden("signal.bw")])
        .args(["-R", &golden("regions.bed")])
        .args(["-o", tmp.path().to_str().unwrap()])
        .args(["-m", "1001", "--bin-size", "50", "-q"])
        .output()
        .unwrap();
    assert!(!out.status.success(), "non-multiple body must fail");
}

#[test]
fn refpoint_zero_flanks_fails() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let out = bin()
        .args(["reference-point", "-S", &golden("signal.bw")])
        .args(["-R", &golden("regions.bed")])
        .args(["-o", tmp.path().to_str().unwrap()])
        .args(["-b", "0", "-a", "0", "-q"])
        .output()
        .unwrap();
    assert!(!out.status.success(), "zero flanks must fail");
}

#[test]
fn hash_grouped_bed_rejected() {
    let bed = tempfile::Builder::new().suffix(".bed").tempfile().unwrap();
    std::fs::write(bed.path(), "#group1\nchr1\t5000\t5200\ta\t0\t+\n").unwrap();
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let out = bin()
        .args(["reference-point", "-S", &golden("signal.bw")])
        .args(["-R", bed.path().to_str().unwrap()])
        .args(["-o", tmp.path().to_str().unwrap()])
        .args(["-b", "100", "-a", "100", "--bin-size", "50", "-q"])
        .output()
        .unwrap();
    assert!(!out.status.success(), "#-grouped BED must be rejected");
}
