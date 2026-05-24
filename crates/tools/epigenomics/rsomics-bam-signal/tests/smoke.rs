use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-signal"))
}

fn golden(n: &str) -> String {
    format!("{}/tests/golden/{}", env!("CARGO_MANIFEST_DIR"), n)
}

#[test]
fn basic_bedgraph_output() {
    let out = bin()
        .arg(golden("small.bam"))
        .args(["-o", "-"])
        .args(["--out-file-format", "bedgraph"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    // Output must be tab-separated bedGraph: chrom start end value.
    assert!(s.contains('\t'));
    let lines: Vec<&str> = s.trim().lines().collect();
    assert!(!lines.is_empty(), "no output lines");
    for line in &lines {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(cols.len(), 4, "each line must have 4 columns: {line}");
        cols[1].parse::<u64>().expect("start must be numeric");
        cols[2].parse::<u64>().expect("end must be numeric");
        cols[3].parse::<f64>().expect("value must be numeric");
    }
}

#[test]
fn bin_size_affects_output() {
    let out50 = bin()
        .arg(golden("small.bam"))
        .args(["-o", "-"])
        .args(["--out-file-format", "bedgraph"])
        .args(["--bin-size", "50"])
        .output()
        .unwrap();
    let out100 = bin()
        .arg(golden("small.bam"))
        .args(["-o", "-"])
        .args(["--out-file-format", "bedgraph"])
        .args(["--bin-size", "100"])
        .output()
        .unwrap();
    assert!(out50.status.success());
    assert!(out100.status.success());
    let lines50 = String::from_utf8_lossy(&out50.stdout)
        .trim()
        .lines()
        .count();
    let lines100 = String::from_utf8_lossy(&out100.stdout)
        .trim()
        .lines()
        .count();
    // Larger bins → fewer (or equal) lines due to merging.
    assert!(
        lines100 <= lines50,
        "larger bins should produce <= lines: 50bp={lines50} 100bp={lines100}"
    );
}

#[test]
fn cpm_normalisation_runs() {
    let out = bin()
        .arg(golden("small.bam"))
        .args(["-o", "-"])
        .args(["--out-file-format", "bedgraph"])
        .args(["--normalize-using", "CPM"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "CPM failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(!s.trim().is_empty(), "CPM produced no output");
}

#[test]
fn rpgc_requires_effective_genome_size() {
    let out = bin()
        .arg(golden("small.bam"))
        .args(["-o", "-"])
        .args(["--out-file-format", "bedgraph"])
        .args(["--normalize-using", "RPGC"])
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "RPGC without --effective-genome-size must fail"
    );
}

#[test]
fn skip_flags_parsed() {
    let out = bin()
        .arg(golden("small.bam"))
        .args(["-o", "-"])
        .args(["--out-file-format", "bedgraph"])
        .args(["--skip-flags", "0x400"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "hex skip-flags failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn bigwig_output_writes_file() {
    let dir = tempfile::tempdir().unwrap();
    let bw_path = dir.path().join("out.bw");
    let out = bin()
        .arg(golden("small.bam"))
        .args(["-o", bw_path.to_str().unwrap()])
        .args(["--out-file-format", "bigwig"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "bigwig failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let meta = std::fs::metadata(&bw_path).expect("bigWig file not created");
    assert!(
        meta.len() > 64,
        "bigWig file too small: {} bytes",
        meta.len()
    );
    // Verify the bigWig magic at byte 0.
    let bytes = std::fs::read(&bw_path).unwrap();
    let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    assert_eq!(magic, 0x888F_FC26, "wrong bigWig magic: 0x{magic:08X}");
}

#[test]
fn bigwig_default_format() {
    // Default format is bigwig — requires an output file, not stdout.
    let out = bin()
        .arg(golden("small.bam"))
        .args(["-o", "-"])
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "default bigwig to stdout must fail with helpful error"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("bigWig"),
        "error message must mention bigWig: {stderr}"
    );
}
