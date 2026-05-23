/// Compatibility test for rsomics-fragment-size.
///
/// The authoritative oracle is `samtools stats`: its `IS` lines are a full-scan
/// insert-size histogram (`IS  <size>  <pairs>  <inward>  <outward>  <other>`),
/// counting each read pair once at the leftmost/first mate of an inward proper
/// pair — exactly the rule this crate applies (paired + proper + mapped + read1,
/// `|TLEN|`). `samtools stats` defaults to `-m 0.99`, which trims the largest
/// 1% of inserts; we pass `-m 1.0` so the full distribution is reported and the
/// comparison is value-exact.
///
/// `deeptools bamPEFragmentSize` is the wrong oracle here: it SAMPLES (~1117
/// fragments by default) rather than full-scanning, so its histogram is a random
/// subset and cannot be compared value-for-value.
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn samtools_available() -> bool {
    Command::new("samtools").arg("--version").output().is_ok()
}

/// Parse our TSV histogram (`size\tcount`, header + non-zero rows) into a map.
fn parse_ours(tsv: &str) -> BTreeMap<u64, u64> {
    let mut hist = BTreeMap::new();
    for line in tsv.lines().skip(1) {
        let mut it = line.split('\t');
        let size: u64 = it.next().unwrap().parse().expect("size");
        let count: u64 = it.next().unwrap().parse().expect("count");
        hist.insert(size, count);
    }
    hist
}

/// Parse `samtools stats` `IS` lines into a size->pairs map, dropping zero rows.
/// Columns: `IS  <size>  <total_pairs>  <inward>  <outward>  <other>`.
fn parse_samtools_is(stats: &str) -> BTreeMap<u64, u64> {
    let mut hist = BTreeMap::new();
    for line in stats.lines() {
        if let Some(rest) = line.strip_prefix("IS\t") {
            let mut it = rest.split('\t');
            let size: u64 = it.next().unwrap().parse().expect("IS size");
            let pairs: u64 = it.next().unwrap().parse().expect("IS pairs");
            if pairs > 0 {
                hist.insert(size, pairs);
            }
        }
    }
    hist
}

#[test]
fn fragment_size_histogram_matches_samtools_stats() {
    if !samtools_available() {
        eprintln!("SKIP: samtools not found; cannot run the insert-size oracle");
        return;
    }

    let bam = golden_dir().join("paired_pe.bam");
    if !bam.exists() {
        eprintln!("SKIP: golden BAM not found at {bam:?}");
        return;
    }

    let binary = env!("CARGO_BIN_EXE_rsomics-fragment-size");
    let ours = Command::new(binary)
        .args([bam.to_str().unwrap(), "-q"])
        .output()
        .expect("rsomics-fragment-size failed to launch");
    assert!(
        ours.status.success(),
        "rsomics-fragment-size exited non-zero"
    );
    let ours_hist = parse_ours(&String::from_utf8(ours.stdout).expect("utf8"));

    let stats = Command::new("samtools")
        .args(["stats", "-m", "1.0", bam.to_str().unwrap()])
        .output()
        .expect("samtools stats failed to launch");
    assert!(stats.status.success(), "samtools stats exited non-zero");
    let sam_hist = parse_samtools_is(&String::from_utf8(stats.stdout).expect("utf8"));

    assert_eq!(
        ours_hist, sam_hist,
        "histogram diverges from samtools stats IS lines"
    );
    assert!(
        ours_hist.values().sum::<u64>() > 0,
        "empty histogram — fixture or selection is wrong"
    );
}

#[test]
fn fragment_size_json_output() {
    let bam = golden_dir().join("paired_pe.bam");
    if !bam.exists() {
        eprintln!("SKIP: golden BAM not found");
        return;
    }

    let binary = env!("CARGO_BIN_EXE_rsomics-fragment-size");
    let output = Command::new(binary)
        .args([bam.to_str().unwrap(), "--json"])
        .output()
        .expect("rsomics-fragment-size failed");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("utf8");
    let json_end = {
        let mut depth = 0i32;
        let mut end = None;
        for (i, c) in stdout.char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(i + 1);
                        break;
                    }
                }
                _ => {}
            }
        }
        end.expect("no closing brace found in JSON output")
    };
    let json: serde_json::Value =
        serde_json::from_str(&stdout[..json_end]).expect("JSON parse failed");
    assert_eq!(json["total_fragments"], 2000);
    let mean = json["mean"].as_f64().unwrap();
    assert!(mean > 0.0, "mean should be positive");
    assert!(json["median"].as_u64().unwrap() > 0);
    assert!(json["mode"].as_u64().unwrap() > 0);
}
