use std::path::PathBuf;
use std::process::{Command, Stdio};

fn ours() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_rsomics-gc-windows"))
}

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/small.fa")
}

fn bedtools_available() -> bool {
    Command::new("bedtools")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

#[test]
fn gc_matches_bedtools_nuc() {
    if !bedtools_available() {
        eprintln!("skipping: bedtools not found");
        return;
    }
    let fa = fixture();

    let our_out = Command::new(ours())
        .args(["-w", "10"])
        .arg(&fa)
        .output()
        .unwrap();
    assert!(our_out.status.success());
    let our_text = String::from_utf8(our_out.stdout).unwrap();

    let bed_lines: Vec<String> = our_text
        .lines()
        .map(|l| {
            let parts: Vec<&str> = l.split('\t').collect();
            format!("{}\t{}\t{}", parts[0], parts[1], parts[2])
        })
        .collect();
    let bed_content = bed_lines.join("\n") + "\n";
    let bed_path = std::env::temp_dir().join("gc_windows_compat.bed");
    std::fs::write(&bed_path, &bed_content).unwrap();

    let bt_out = Command::new("bedtools")
        .args([
            "nuc",
            "-fi",
            fa.to_str().unwrap(),
            "-bed",
            bed_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(bt_out.status.success());
    let bt_text = String::from_utf8(bt_out.stdout).unwrap();

    let our_gc: Vec<f64> = our_text
        .lines()
        .filter_map(|l| l.split('\t').nth(3)?.parse().ok())
        .collect();
    let bt_gc: Vec<f64> = bt_text
        .lines()
        .skip(1)
        .filter_map(|l| l.split('\t').nth(4)?.parse().ok())
        .collect();

    assert_eq!(
        our_gc.len(),
        bt_gc.len(),
        "window count mismatch: {} vs {}",
        our_gc.len(),
        bt_gc.len()
    );

    for (i, (ours, theirs)) in our_gc.iter().zip(bt_gc.iter()).enumerate() {
        assert!(
            (ours - theirs).abs() < 0.001,
            "window {i}: GC {ours:.4} vs bedtools {theirs:.6}"
        );
    }

    let _ = std::fs::remove_file(&bed_path);
}
