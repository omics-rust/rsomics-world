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

/// ampliconclip's byte-exact compat is version-pinned to samtools >= 1.23. The
/// clip-coordinate and tag-deletion semantics have evolved across releases (the
/// main CI runs apt samtools 1.19.2), so gate on version to avoid a version-skew
/// false-fail — the same pattern fixmate/fastp compat use. The authoritative
/// compat run is mac samtools 1.23.1 and the 4090 conda samtools.
fn samtools_compat_ready() -> bool {
    let Ok(out) = Command::new("samtools").arg("--version").output() else {
        return false;
    };
    if !out.status.success() {
        return false;
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let num = stdout
        .lines()
        .next()
        .unwrap_or("")
        .split_whitespace()
        .nth(1)
        .unwrap_or("");
    let mut it = num.split('.');
    let major: u32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor: u32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    if major == 1 && minor >= 23 {
        return true;
    }
    eprintln!(
        "SKIP ampliconclip compat: samtools {num} (need >= 1.23; clip semantics differ on older releases)"
    );
    false
}

fn run_ok(cmd: &mut Command) {
    let status = cmd.status().unwrap();
    assert!(status.success(), "command failed: {cmd:?}");
}

/// Full per-record SAM line (all columns including tags), one String per record.
/// The header is dropped — `@PG`/`SO` lines legitimately differ (samtools writes
/// its own program name); the contract here is byte-exact ALIGNMENT records.
fn sam_records(bam: &Path) -> Vec<String> {
    let out = Command::new("samtools")
        .args(["view"])
        .arg(bam)
        .output()
        .unwrap();
    assert!(out.status.success(), "samtools view failed on {bam:?}");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_owned)
        .collect()
}

/// Build the coordinate-sorted input BAM once per test from the golden SAM.
fn make_input(dir: &Path) -> PathBuf {
    let in_bam = dir.join("in.bam");
    let unsorted = dir.join("unsorted.bam");
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
    in_bam
}

/// Run ours and samtools with the same extra flags and assert the alignment
/// records match byte-for-byte (every column, every tag).
fn assert_mode(label: &str, in_bam: &Path, dir: &Path, extra: &[&str]) {
    let bed = golden_bed();

    let st_out = dir.join(format!("st_{label}.bam"));
    let mut st = Command::new("samtools");
    st.args(["ampliconclip", "--no-PG", "-b"])
        .arg(&bed)
        .args(extra)
        .arg(in_bam)
        .arg("-o")
        .arg(&st_out);
    run_ok(&mut st);

    let our_out = dir.join(format!("ours_{label}.bam"));
    let mut our = ours();
    our.args(["--no-PG", "-b"])
        .arg(&bed)
        .args(extra)
        .arg(in_bam)
        .arg("-o")
        .arg(&our_out);
    run_ok(&mut our);

    let st_records = sam_records(&st_out);
    let our_records = sam_records(&our_out);

    assert_eq!(
        st_records.len(),
        our_records.len(),
        "[{label}] record count mismatch: samtools={} ours={}",
        st_records.len(),
        our_records.len()
    );

    for (idx, (st, our)) in st_records.iter().zip(our_records.iter()).enumerate() {
        assert_eq!(
            st, our,
            "[{label}] record {idx} mismatch:\n  samtools: {st}\n  ours:     {our}"
        );
    }
}

#[test]
fn ampliconclip_matches_samtools() {
    if !samtools_compat_ready() {
        eprintln!("skipping: samtools not available or too old");
        return;
    }

    let dir = std::env::temp_dir().join("rsomics-bam-ampliconclip-compat");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let in_bam = make_input(&dir);

    // Default soft clip 5' only, then every implemented flag and a few combos.
    assert_mode("soft_default", &in_bam, &dir, &[]);
    assert_mode("hard", &in_bam, &dir, &["--hard-clip"]);
    assert_mode("both_soft", &in_bam, &dir, &["--both-ends"]);
    assert_mode("both_hard", &in_bam, &dir, &["--both-ends", "--hard-clip"]);
    assert_mode("strand_soft", &in_bam, &dir, &["--strand"]);
    assert_mode("strand_hard", &in_bam, &dir, &["--strand", "--hard-clip"]);
    assert_mode("strand_both", &in_bam, &dir, &["--strand", "--both-ends"]);
    assert_mode("keep_tag", &in_bam, &dir, &["--keep-tag"]);
    assert_mode("fail", &in_bam, &dir, &["--fail"]);
    assert_mode("clipped", &in_bam, &dir, &["--clipped"]);
    assert_mode("no_excluded", &in_bam, &dir, &["--no-excluded"]);
    assert_mode("tol0", &in_bam, &dir, &["--tolerance", "0"]);
    assert_mode("tol20", &in_bam, &dir, &["--tolerance", "20"]);
    assert_mode("filter_len", &in_bam, &dir, &["--filter-len", "50"]);
    assert_mode("fail_len", &in_bam, &dir, &["--fail-len", "50"]);
    assert_mode("unmap_len", &in_bam, &dir, &["--unmap-len", "90"]);
    assert_mode(
        "combo",
        &in_bam,
        &dir,
        &[
            "--both-ends",
            "--hard-clip",
            "--strand",
            "--fail-len",
            "30",
            "--keep-tag",
            "--unmap-len",
            "40",
        ],
    );
}
