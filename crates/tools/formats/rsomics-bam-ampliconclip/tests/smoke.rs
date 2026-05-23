use std::path::{Path, PathBuf};
use std::process::Command;

fn ours() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-ampliconclip"))
}

fn golden_sam() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/reads.sam")
}

fn golden_bed() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/primers.bed")
}

fn samtools_available() -> bool {
    Command::new("samtools")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn run_ok(cmd: &mut Command) {
    let status = cmd.status().unwrap();
    assert!(status.success(), "command failed: {cmd:?}");
}

fn sam_records(bam: &Path) -> Vec<Vec<String>> {
    let out = Command::new("samtools")
        .args(["view"])
        .arg(bam)
        .output()
        .unwrap();
    assert!(out.status.success());
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| l.split('\t').map(str::to_owned).collect())
        .collect()
}

/// End-to-end: build the golden input, run ours (default soft clip), and check
/// the forward read is soft-clipped (20S80M, POS advanced) and the no-match read
/// passes through untouched. This exercises the binary without asserting
/// byte-equality vs samtools — that is `compat.rs`'s job.
#[test]
fn soft_clip_end_to_end() {
    if !samtools_available() {
        eprintln!("skipping smoke: samtools not on PATH");
        return;
    }

    let dir = std::env::temp_dir().join("rsomics-bam-ampliconclip-smoke");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let unsorted = dir.join("unsorted.bam");
    let in_bam = dir.join("in.bam");
    run_ok(
        Command::new("samtools")
            .args(["view", "-b", "-o"])
            .arg(&unsorted)
            .arg(golden_sam()),
    );
    run_ok(
        Command::new("samtools")
            .args(["sort", "-o"])
            .arg(&in_bam)
            .arg(&unsorted),
    );

    let out_bam = dir.join("out.bam");
    run_ok(
        ours()
            .args(["--no-PG", "-b"])
            .arg(golden_bed())
            .arg(&in_bam)
            .arg("-o")
            .arg(&out_bam),
    );

    let records = sam_records(&out_bam);

    let fwd1 = records
        .iter()
        .find(|r| r[0] == "read_fwd1")
        .expect("read_fwd1 present");
    // Primer chr1:[100,120) over a read starting at ref 100 → 20 ref bases clip.
    assert_eq!(fwd1[5], "20S80M", "read_fwd1 CIGAR");
    assert_eq!(fwd1[3], "121", "read_fwd1 POS advances over the clip");
    // NM tag is deleted on a clipped read by default.
    assert!(
        !fwd1.iter().any(|f| f.starts_with("NM:")),
        "NM tag should be deleted on a clipped read"
    );

    let nomatch = records
        .iter()
        .find(|r| r[0] == "read_nomatch")
        .expect("read_nomatch present");
    // No primer overlaps → untouched, NM tag retained.
    assert_eq!(nomatch[5], "50M", "read_nomatch CIGAR untouched");
    assert!(
        nomatch.iter().any(|f| f.starts_with("NM:")),
        "NM tag retained on an unclipped read"
    );
}
