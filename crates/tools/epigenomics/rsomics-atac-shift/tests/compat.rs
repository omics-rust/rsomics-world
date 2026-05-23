/// Compatibility test against `deeptools alignmentSieve --ATACshift` v3.5.6.
///
/// Checks field-by-field match (name, flag, tid, pos, cigar, tlen, next_pos)
/// between our output and the deeptools golden fixture. Byte-exact BAM
/// comparison is not feasible because deeptools writes different @PG headers;
/// instead, we compare the semantically meaningful record fields.
///
/// The test self-skips when:
/// - `alignmentSieve` is not on PATH (deeptools not installed).
/// - The fixture BAM is absent (stripped test env).
use std::path::PathBuf;
use std::process::Command;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn alignmentsieve_available() -> bool {
    Command::new("alignmentSieve")
        .arg("--version")
        .output()
        .is_ok()
}

fn samtools_available() -> bool {
    Command::new("samtools").arg("--version").output().is_ok()
}

/// Parse `samtools view` text output into (name, flag, pos_1based, cigar, tlen, next_pos_1based) tuples,
/// sorted by (name, flag) so output-order differences between tools don't produce false mismatches.
fn parse_sam_fields_sorted(output: &str) -> Vec<(String, u16, i64, String, i32, i64)> {
    let mut records: Vec<(String, u16, i64, String, i32, i64)> = output
        .lines()
        .filter(|l| !l.starts_with('@'))
        .map(|l| {
            let fields: Vec<&str> = l.split('\t').collect();
            let name = fields[0].to_string();
            let flag: u16 = fields[1].parse().unwrap();
            let pos: i64 = fields[3].parse().unwrap(); // 1-based SAM
            let cigar = fields[5].to_string();
            let tlen: i32 = fields[8].parse().unwrap();
            let next_pos: i64 = fields[7].parse().unwrap(); // 1-based
            (name, flag, pos, cigar, tlen, next_pos)
        })
        .collect();
    records.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    records
}

fn bam_to_sam_text(bam: &std::path::Path) -> String {
    let out = Command::new("samtools")
        .args(["view", "-h"])
        .arg(bam)
        .output()
        .expect("samtools view failed");
    assert!(out.status.success(), "samtools view exited non-zero");
    String::from_utf8(out.stdout).expect("samtools output not UTF-8")
}

fn run_compat_check(input_bam: &std::path::Path) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let our_out = tmp.path().join("our_shifted.bam");
    let dt_out = tmp.path().join("dt_shifted.bam");

    let binary = env!("CARGO_BIN_EXE_rsomics-atac-shift");
    let status = Command::new(binary)
        .args([
            input_bam.to_str().unwrap(),
            "-o",
            our_out.to_str().unwrap(),
            "-q",
        ])
        .status()
        .expect("rsomics-atac-shift failed to launch");
    assert!(status.success(), "rsomics-atac-shift exited non-zero");

    let dt_status = Command::new("alignmentSieve")
        .args([
            "-b",
            input_bam.to_str().unwrap(),
            "-o",
            dt_out.to_str().unwrap(),
            "--ATACshift",
            "--numberOfProcessors",
            "1",
        ])
        .status()
        .expect("alignmentSieve failed to launch");
    assert!(dt_status.success(), "alignmentSieve exited non-zero");

    let our_sam = bam_to_sam_text(&our_out);
    let dt_sam = bam_to_sam_text(&dt_out);

    let our_records = parse_sam_fields_sorted(&our_sam);
    let dt_records = parse_sam_fields_sorted(&dt_sam);

    assert_eq!(
        our_records.len(),
        dt_records.len(),
        "record count mismatch: ours={} deeptools={}",
        our_records.len(),
        dt_records.len()
    );

    for (i, (ours, dt)) in our_records.iter().zip(dt_records.iter()).enumerate() {
        assert_eq!(
            ours.0, dt.0,
            "record {i}: name mismatch: {:?} vs {:?}",
            ours.0, dt.0
        );
        assert_eq!(
            ours.1, dt.1,
            "record {i} ({}): flag mismatch: {} vs {}",
            ours.0, ours.1, dt.1
        );
        assert_eq!(
            ours.2, dt.2,
            "record {i} ({}): POS mismatch: {} vs {} (deeptools)",
            ours.0, ours.2, dt.2
        );
        assert_eq!(
            ours.3, dt.3,
            "record {i} ({}): CIGAR mismatch: {:?} vs {:?} (deeptools)",
            ours.0, ours.3, dt.3
        );
        assert_eq!(
            ours.4, dt.4,
            "record {i} ({}): TLEN mismatch: {} vs {}",
            ours.0, ours.4, dt.4
        );
        assert_eq!(
            ours.5, dt.5,
            "record {i} ({}): PNEXT mismatch: {} vs {}",
            ours.0, ours.5, dt.5
        );
    }
}

#[test]
fn atac_shift_matches_deeptools() {
    if !alignmentsieve_available() {
        eprintln!("SKIP: alignmentSieve not found (deeptools not installed)");
        return;
    }
    if !samtools_available() {
        eprintln!("SKIP: samtools not found");
        return;
    }

    let fixtures = fixture_dir();
    let input_bam = fixtures.join("atac_small.bam");
    if !input_bam.exists() {
        eprintln!("SKIP: fixture BAM not found at {:?}", input_bam);
        return;
    }

    run_compat_check(&input_bam);
}

/// Representative compat test using a ~3700-record fixture derived from bench.bam
/// (chr1:1-30000, ~65% soft-clipped reads). This exercises the CIGAR class that
/// the 50M-only atac_small.bam fixture cannot catch: reads with leading or trailing
/// soft clips (e.g. `3S97M`) where deeptools uses `query_alignment_end` (query
/// coordinate) rather than the reference-consuming CIGAR span to compute `end`,
/// producing a different new-CIGAR length (96M vs 93M for `3S97M` with +4 shift).
#[test]
fn atac_shift_softclip_compat() {
    if !alignmentsieve_available() {
        eprintln!("SKIP: alignmentSieve not found (deeptools not installed)");
        return;
    }
    if !samtools_available() {
        eprintln!("SKIP: samtools not found");
        return;
    }

    let fixtures = fixture_dir();
    let input_bam = fixtures.join("bench_small.bam");
    if !input_bam.exists() {
        eprintln!("SKIP: bench_small.bam not found");
        return;
    }

    run_compat_check(&input_bam);
}
