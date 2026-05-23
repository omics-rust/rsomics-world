use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rsomics-bam-mpileup"))
}

fn golden_bam() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/coord_sorted.bam")
}

/// The crate runs end-to-end on the committed coordinate-sorted golden BAM and
/// produces 6-column pileup lines (chrom, 1-based pos, ref base, depth, bases,
/// quals) — no samtools needed. This guards the engine + encoder against a build
/// that compiles but emits nothing.
#[test]
fn mpileup_smoke_no_ref() {
    let out = bin().arg("-t1").arg(golden_bam()).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = text.lines().collect();
    assert!(!lines.is_empty(), "no pileup output");

    for line in &lines {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(cols.len(), 6, "expected 6 columns, got line: {line}");
        // ref column is N without a reference.
        assert_eq!(cols[2], "N", "no-ref ref column should be N: {line}");
        let depth: usize = cols[3].parse().expect("depth must be an integer");
        // base/qual columns are `*` only at zero depth (not produced without -a).
        assert!(depth > 0, "default mode should not emit zero-depth rows");
    }

    // The fixture has an insertion (8M2I8M) and a deletion (5M3D5M); the bases
    // column must carry both `+`/`-` indel notation and a `*` deleted placeholder
    // somewhere across the output.
    assert!(
        text.contains('+'),
        "expected an insertion (+) in the pileup"
    );
    assert!(text.contains('-'), "expected a deletion (-) in the pileup");
    assert!(
        text.contains('*'),
        "expected a deleted-base (*) in the pileup"
    );
    // Head markers (^) and a tail marker ($) appear in a small fixture.
    assert!(text.contains('^'), "expected a read-start marker (^)");
    assert!(text.contains('$'), "expected a read-end marker ($)");
}

/// `-f ref -B` switches the ref column to the FASTA base and ref-matching bases
/// to `.`/`,`.
#[test]
fn mpileup_smoke_ref() {
    let ref_fa = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/ref.fa");
    let out = bin()
        .arg("-t1")
        .arg("-f")
        .arg(&ref_fa)
        .arg("-B")
        .arg(golden_bam())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains('.') || text.contains(','),
        "expected ref-match dots"
    );
    // ref column should not be a literal N for a covered position on chr1.
    let first = text.lines().next().unwrap();
    let cols: Vec<&str> = first.split('\t').collect();
    assert_ne!(
        cols[2], "N",
        "ref column should be a FASTA base, got: {first}"
    );
}

/// Reference-aware without `-B` is refused loudly (BAQ unimplemented).
#[test]
fn mpileup_ref_without_b_errors() {
    let ref_fa = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/ref.fa");
    let out = bin()
        .arg("-t1")
        .arg("-f")
        .arg(&ref_fa)
        .arg(golden_bam())
        .output()
        .unwrap();
    assert!(!out.status.success(), "expected failure without -B");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("BAQ"), "expected BAQ error, got: {err}");
}
