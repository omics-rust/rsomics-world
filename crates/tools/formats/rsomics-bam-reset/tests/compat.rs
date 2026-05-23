use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn ours() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-reset"))
}

fn golden() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/small.bam")
}

/// Reset's record output is byte-exact only against samtools' own decode/encode,
/// so gate to the version this crate was developed against. A different samtools
/// can shift formatting (e.g. the `seq_nt16` table or aux ordering) — loud-skip
/// rather than fail spuriously.
fn samtools_is_1_23() -> bool {
    let out = match Command::new("samtools").arg("version").output() {
        Ok(o) if o.status.success() => o,
        _ => return false,
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()
        .is_some_and(|l| l.contains("samtools 1.23"))
}

fn run_ok(cmd: &mut Command) {
    let status = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert!(status.success(), "command failed: {cmd:?}");
}

/// Every record's full SAM line (all 11 mandatory fields + aux), sorted by QNAME.
/// Byte-exact comparison of FLAG/RNAME/POS/MAPQ/CIGAR/RNEXT/PNEXT/TLEN/SEQ/QUAL
/// plus the aux tail.
fn record_lines(bam: &Path) -> Vec<String> {
    let out = Command::new("samtools")
        .arg("view")
        .arg(bam)
        .output()
        .unwrap();
    assert!(out.status.success(), "samtools view failed on {bam:?}");
    let mut lines: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_owned)
        .collect();
    lines.sort();
    lines
}

/// Header lines except the provenance `@PG` lines that differ by construction:
/// the reset PG (ours: `PN:rsomics-bam-reset`, theirs: `CL:samtools reset …`)
/// and the `samtools view -H` PG that the comparison itself appends (whose `ID`
/// suffix and `CL` carry the temp output path). Keeps @HD, @SQ, @RG and the
/// carried-over input @PG chain — the lines that must match.
fn stable_header_lines(bam: &Path) -> Vec<String> {
    let out = Command::new("samtools")
        .args(["view", "-H"])
        .arg(bam)
        .output()
        .unwrap();
    assert!(out.status.success());
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| {
            !(l.starts_with("@PG")
                && (l.contains("PN:rsomics-bam-reset")
                    || l.contains("CL:samtools reset")
                    || l.contains("CL:samtools view")))
        })
        .map(str::to_owned)
        .collect()
}

fn compare(args_ours: &[&str], args_samtools: &[&str], label: &str) {
    let dir = std::env::temp_dir().join(format!("rsomics-bam-reset-compat-{label}"));
    std::fs::create_dir_all(&dir).unwrap();
    let our_out = dir.join("ours.bam");
    let st_out = dir.join("samtools.bam");

    run_ok(ours().arg(golden()).args(args_ours).arg("-o").arg(&our_out));
    run_ok(
        Command::new("samtools")
            .arg("reset")
            .args(args_samtools)
            .arg("-o")
            .arg(&st_out)
            .arg(golden()),
    );

    assert_eq!(
        record_lines(&our_out),
        record_lines(&st_out),
        "[{label}] record fields must match samtools reset byte-exact"
    );
    assert_eq!(
        stable_header_lines(&our_out),
        stable_header_lines(&st_out),
        "[{label}] @HD/@RG/input-@PG header lines must match samtools reset"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn reset_default_matches_samtools() {
    if !samtools_is_1_23() {
        eprintln!("SKIP: samtools 1.23 not found");
        return;
    }
    compare(&[], &[], "default");
}

#[test]
fn reset_no_rg_matches_samtools() {
    if !samtools_is_1_23() {
        eprintln!("SKIP: samtools 1.23 not found");
        return;
    }
    compare(&["--no-RG"], &["--no-RG"], "no-rg");
}

#[test]
fn reset_keep_tag_matches_samtools() {
    if !samtools_is_1_23() {
        eprintln!("SKIP: samtools 1.23 not found");
        return;
    }
    compare(
        &["--keep-tag", "RG,BC"],
        &["--keep-tag", "RG,BC"],
        "keep-tag",
    );
}

#[test]
fn reset_remove_tag_matches_samtools() {
    if !samtools_is_1_23() {
        eprintln!("SKIP: samtools 1.23 not found");
        return;
    }
    compare(&["-x", "XS"], &["-x", "XS"], "remove-tag");
}

#[test]
fn reset_dupflag_matches_samtools() {
    if !samtools_is_1_23() {
        eprintln!("SKIP: samtools 1.23 not found");
        return;
    }
    compare(&["--dupflag"], &["--dupflag"], "dupflag");
}

#[test]
fn reset_no_pg_matches_samtools() {
    if !samtools_is_1_23() {
        eprintln!("SKIP: samtools 1.23 not found");
        return;
    }
    compare(&["--no-PG"], &["--no-PG"], "no-pg");
}
