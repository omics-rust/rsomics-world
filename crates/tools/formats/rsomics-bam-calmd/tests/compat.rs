use std::path::{Path, PathBuf};
use std::process::Command;

fn ours() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-calmd"))
}

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

/// calmd's MD/NM core (`bam_fillmd1_core`) has been byte-stable since 2020 — the
/// MD run-length string, the `^`-deletion form, the nibble-level match test, and
/// the "rewrite the tag only when its value differs (and then move it to the end
/// of the aux block)" rule are identical in samtools 1.19 (the CI apt build) and
/// 1.23 (the dev mac build). So gate on `>= 1.19` rather than pinning to 1.23:
/// both produce the same calmd records and the test runs for real on CI instead
/// of always skipping. Sub-1.19 / missing samtools is skipped, not failed.
fn samtools_compat_ready() -> bool {
    let Ok(out) = Command::new("samtools").arg("--version").output() else {
        eprintln!("SKIP calmd compat: samtools not found");
        return false;
    };
    if !out.status.success() {
        eprintln!("SKIP calmd compat: `samtools --version` failed");
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
    if major > 1 || (major == 1 && minor >= 19) {
        return true;
    }
    eprintln!("SKIP calmd compat: samtools {num} (need >= 1.19)");
    false
}

fn run_ok(cmd: &mut Command) {
    let status = cmd.status().unwrap();
    assert!(status.success(), "command failed: {cmd:?}");
}

/// Every alignment record as SAM text, in stream order. This is the byte-exact
/// check: it covers FLAG, all positional fields, SEQ (so `-e` `=`-conversion is
/// caught), and every aux tag in order — including the NM/MD ordering rule.
fn records(bam: &Path) -> String {
    let out = Command::new("samtools")
        .arg("view")
        .arg(bam)
        .output()
        .unwrap();
    assert!(out.status.success(), "samtools view failed on {bam:?}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Stage a coordinate-sorted BAM and the indexed reference into `dir`, then
/// return their paths. The reference `.fai` is copied alongside `ref.fa` so the
/// indexed-FASTA reader (ours) and `fai_load` (samtools) both find it.
fn stage(dir: &Path) -> (PathBuf, PathBuf) {
    let _ = std::fs::create_dir_all(dir);
    let gold = golden_dir();

    let reference = dir.join("ref.fa");
    std::fs::copy(gold.join("ref.fa"), &reference).unwrap();
    std::fs::copy(gold.join("ref.fa.fai"), dir.join("ref.fa.fai")).unwrap();

    let bam = dir.join("in.bam");
    let unsorted = dir.join("unsorted.bam");
    {
        let f = std::fs::File::create(&unsorted).unwrap();
        run_ok(
            Command::new("samtools")
                .args(["view", "-b"])
                .arg(gold.join("calmd_in.sam"))
                .stdout(f),
        );
    }
    run_ok(
        Command::new("samtools")
            .args(["sort", "-o"])
            .arg(&bam)
            .arg(&unsorted),
    );

    (bam, reference)
}

/// Default calmd (recompute MD + NM) must be byte-exact against `samtools calmd`.
#[test]
fn calmd_default_matches_samtools() {
    if !samtools_compat_ready() {
        return;
    }
    let dir = std::env::temp_dir().join("rsomics-bam-calmd-compat-default");
    let (bam, reference) = stage(&dir);

    let sm = dir.join("samtools.bam");
    {
        let f = std::fs::File::create(&sm).unwrap();
        run_ok(
            Command::new("samtools")
                .args(["calmd", "-b", "--no-PG"])
                .arg(&bam)
                .arg(&reference)
                .stdout(f)
                .stderr(std::process::Stdio::null()),
        );
    }

    let our = dir.join("ours.bam");
    run_ok(
        ours()
            .arg(&bam)
            .arg(&reference)
            .arg("-o")
            .arg(&our)
            .stderr(std::process::Stdio::null()),
    );

    assert_eq!(
        records(&our),
        records(&sm),
        "every output record must be byte-exact against samtools calmd (default MD+NM)"
    );
}

/// `-e` (convert reference-matching bases to `=`) must be byte-exact too: it
/// rewrites SEQ on matches while leaving mismatch and inserted bases intact.
#[test]
fn calmd_use_equal_matches_samtools() {
    if !samtools_compat_ready() {
        return;
    }
    let dir = std::env::temp_dir().join("rsomics-bam-calmd-compat-equal");
    let (bam, reference) = stage(&dir);

    let sm = dir.join("samtools.bam");
    {
        let f = std::fs::File::create(&sm).unwrap();
        run_ok(
            Command::new("samtools")
                .args(["calmd", "-b", "-e", "--no-PG"])
                .arg(&bam)
                .arg(&reference)
                .stdout(f)
                .stderr(std::process::Stdio::null()),
        );
    }

    let our = dir.join("ours.bam");
    run_ok(
        ours()
            .arg(&bam)
            .arg(&reference)
            .arg("-e")
            .arg("-o")
            .arg(&our)
            .stderr(std::process::Stdio::null()),
    );

    assert_eq!(
        records(&our),
        records(&sm),
        "every output record must be byte-exact against samtools calmd -e"
    );
}
