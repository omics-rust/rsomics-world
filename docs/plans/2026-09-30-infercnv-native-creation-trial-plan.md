# Native raw-creation trial implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans. Root observes real RED/GREEN; bounded independent reviewers do not publish or stage.

**Goal:** Add a private actual-factory raw-ingestion executable with exact warm/measured exports and correctly scoped counters; no performance acceptance here.

**Architecture:** Reuse the frozen creation comparison/export helper and CPU/RSS support. Keep input selection/config/metadata outside the timed public create_input call. Drop warm/expected state before baseline; sample RSS with returned state live before oracle reload/export/format. An external controller will authenticate inputs/source/binary before and after 32 fresh processes.

**Tech stack:** Rust 1.91, existing nix/flate2 dev dependencies, current infercnv-measurement feature; hosted four-native CI. No new dependencies, public API, production changes or local build.

**Spec:** docs/plans/2026-09-30-infercnv-raw-ingestion-measurement-design.md and .autopilot/oracles/infercnv-ingestion-measurement-input-2026-09-30.json.

## Prerequisite and preserved state

- Accept clean green-12 only after original exact-head four-native APIs/logs/artifacts and source inventory audit. Until then do not modify the owning product's three frozen support files.
- Owning product: /Volumes/KIOXIA/Documents/omics-rust/rsomics-sc. CARGO_HOME=/Volumes/KIOXIA/Developments/cargo-home, target=/Volumes/KIOXIA/Developments/cargo-target, TMPDIR=/Volumes/KIOXIA/Developments/tmp. APFS >80% forbids local Cargo/R/build/install.
- Source/evidence archives stay on external disks. Preserve polluted red-7/green-11 and clean green-12; no overwrite/deletion.
- Source production, Cargo.lock, existing tests/bench/support and accepted creation helper remain byte-identical.
- No source-substring tests, fake factory, input regeneration, new foundation or public CLI.

## Frozen proposal and source review

Root read all complete files. Independent timer review found no critical or additional important timer/counter defect. Two textual source-order tests were removed; they were spelling tests, not runtime behavior. Lifecycle sequence is independently source-reviewed and frozen, not dynamically proven by these tests.

| Proposal | Final path | SHA256 |
| --- | --- | --- |
| contract.rs | tests/cnv_ingestion_trial_contract.rs | d8f291c0553a6e67da990c0f3a3aba76be7155cc46d5c1d405ddedb2c942cab4 |
| trial.rs | benches/ingestion_trial/mod.rs | cee49f464663367d1c7621649c8087df4f0180966ee046eaa3a26ef5bc80f18e |
| cnv_ingestion_matched.rs | benches/cnv_ingestion_matched.rs | 90fa9723cf36267aeae5f22638062ae894100728cd3485edfd45e4addf458019 |

CLI: ARTIFACT_ROOT plain|gzip NEW_OUTPUT_DIR. Fixed accepted-receipt paths; references from expected ordered maps; chrX/chrY/chrM exclusions; depth [1, Inf]; numeric 256 MiB/decoded record 1 MiB. Runtime declarations do not authenticate inputs. Only the later controller performs original API/ZIP/hash/gzip equivalence and pre/post source/executable checks.

Literal key/value metrics have exactly12 keys: schema_version, measurement, implementation, input_case, preparation_wall_seconds, region_wall_seconds, region_user_seconds, region_system_seconds, region_child_user_seconds, region_child_system_seconds, baseline_rss_bytes, returned_rss_bytes. I/O is a post-export lifetime procfs snapshot, not factory delta. Bracketing CPU/RSS instrumentation skew and allocator-retained pages remain explicit.

### Task 1: Real missing-module RED

**Files:** Add complete tests/cnv_ingestion_trial_contract.rs and append only this Cargo target:

```toml
[[test]]
name = "cnv_ingestion_trial_contract"
path = "tests/cnv_ingestion_trial_contract.rs"
required-features = ["infercnv-measurement"]
```

- [x] Add only the complete test below; helper/bench remain absent. Standalone external rustfmt may run, Cargo may not.
- [x] Add default-false ingestion_trial workflow input/env; require ingestion_measurement=true and measurement=true. Preserve the source guard and all old selectors. Add dedicated debug/release trial target before old support tests; release and bench compile only for !expected_red.
- [x] Freeze red-8 using COPYFILE_DISABLE=1 tar --no-xattrs --format=ustar and exact sorted README/manifest/archive checksums. Compare all previous files except Cargo.toml; only new test may be added. Generic source guard must pass before dispatch.
- [x] Commit only world workflow/plan/ledger/red-8; push and wait exact-head Control plane. Dispatch expected_red=true, measurement=true, ingestion_measurement=true, ingestion_trial=true, oracle_receipt=none.
- [x] Preserve run/jobs/artifact APIs, original ZIP/logs. Accept only the exact couldn't-read tests/../benches/ingestion_trial/mod.rs OS-error2 failure after source/dependency success. No generic failed compilation or failed packaging is RED.

Complete test:

```rust
#[allow(
    dead_code,
    reason = "contract tests exercise private trial interfaces without its process entry point"
)]
#[path = "../benches/ingestion_trial/mod.rs"]
mod trial;

#[cfg(target_os = "linux")]
use std::collections::BTreeMap;
#[cfg(target_os = "linux")]
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use trial::{Args, CpuSample, InputCase, Inputs, RegionMetrics};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct TempRoot(PathBuf);

impl TempRoot {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "rsomics-ingestion-trial-contract-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn cpu(user_micros: u64, system_micros: u64) -> CpuSample {
    CpuSample {
        user_micros,
        system_micros,
    }
}

#[test]
fn only_three_positional_arguments_are_accepted() {
    let valid = vec!["artifact".into(), "plain".into(), "output".into()];
    let parsed = Args::parse(&valid).unwrap();
    assert_eq!(parsed.artifact_root, Path::new("artifact"));
    assert_eq!(parsed.output, Path::new("output"));
    assert_eq!(parsed.case, InputCase::Plain);
    assert!(Args::parse(&valid[..2]).is_err());
    let mut extra = valid;
    extra.push("pair-index".into());
    assert!(Args::parse(&extra).is_err());
    assert!(Args::parse(&["".into(), "plain".into(), "output".into()]).is_err());
    assert!(Args::parse(&["artifact".into(), "plain".into(), "".into()]).is_err());
}

#[test]
fn case_names_are_literal_and_closed() {
    assert_eq!(
        Args::parse(&["root".into(), "gzip".into(), "out".into()])
            .unwrap()
            .case,
        InputCase::Gzip
    );
    for invalid in ["GZIP", "plain ", "subset", "", "r-first"] {
        assert!(Args::parse(&["root".into(), invalid.into(), "out".into()]).is_err());
    }
}

#[test]
fn paths_match_the_current_frozen_receipt_layout() {
    let root = Path::new("artifact");
    let plain = Inputs::new(root, InputCase::Plain);
    let gzip = Inputs::new(root, InputCase::Gzip);
    assert_eq!(
        plain.counts,
        root.join("shipped-bundle/inputs/canonical-full.tsv")
    );
    assert_eq!(
        gzip.counts,
        root.join("ingestion-witness/shipped_full/counts.tsv.gz")
    );
    assert_eq!(
        plain.positions,
        root.join("shipped-bundle/inputs/gencode_downsampled.EXAMPLE_ONLY_DONT_REUSE.txt")
    );
    assert_eq!(
        plain.annotations,
        root.join("shipped-bundle/inputs/oligodendroglioma_annotations_downsampled.txt")
    );
    assert_eq!(plain.expected_full, root.join("shipped-bundle/full"));
    assert_eq!(
        plain.expected_maps,
        root.join("ingestion-witness/shipped_full/plain/maps.tsv")
    );
    assert_eq!(plain.plain_counts, gzip.plain_counts);
    assert_eq!(plain.positions, gzip.positions);
    assert_eq!(plain.annotations, gzip.annotations);
    assert_eq!(plain.expected_full, gzip.expected_full);
    assert_eq!(plain.expected_maps, gzip.expected_maps);
}

#[test]
fn configuration_retains_requested_references_and_bounded_limits() {
    let config = trial::creation_config(vec!["r_b".into(), "r_a".into()]);
    assert_eq!(config.reference_groups, ["r_b", "r_a"]);
    assert_eq!(config.excluded_chromosomes, ["chrX", "chrY", "chrM"]);
    assert_eq!(config.min_counts_per_cell, 1.0);
    assert_eq!(config.max_counts_per_cell, f64::INFINITY);
    assert_eq!(config.max_numeric_bytes, 268_435_456);
    assert_eq!(config.max_record_bytes, 1_048_576);
}

#[test]
fn input_case_must_agree_with_actual_magic() {
    let root = TempRoot::new();
    let plain = root.0.join("plain.tsv");
    let gzip = root.0.join("compressed.tsv");
    fs::write(&plain, b"gene\tc\nG\t1\n").unwrap();
    fs::write(&gzip, [0x1f, 0x8b, 0]).unwrap();
    InputCase::Plain.check_magic(&plain).unwrap();
    InputCase::Gzip.check_magic(&gzip).unwrap();
    assert!(InputCase::Gzip.check_magic(&plain).is_err());
    assert!(InputCase::Plain.check_magic(&gzip).is_err());
    fs::write(&plain, b"g").unwrap();
    assert!(InputCase::Plain.check_magic(&plain).is_err());
}

#[test]
fn region_cpu_deltas_have_explicit_seconds_and_no_child_work() {
    let metrics = RegionMetrics::checked(
        0.25,
        cpu(1_000_000, 3),
        cpu(1_500_000, 8),
        cpu(100, 20),
        cpu(100, 20),
        4096,
    )
    .unwrap();
    assert_eq!(metrics.wall_seconds, 0.25);
    assert_eq!(metrics.user_seconds, 0.5);
    assert_eq!(metrics.system_seconds, 0.000_005);
    assert_eq!(metrics.child_user_seconds, 0.0);
    assert_eq!(metrics.child_system_seconds, 0.0);
    assert_eq!(metrics.returned_rss_bytes, 4096);
}

#[test]
fn invalid_wall_or_rss_cannot_be_a_sample() {
    for wall in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(
            RegionMetrics::checked(wall, cpu(0, 0), cpu(0, 0), cpu(0, 0), cpu(0, 0), 1).is_err()
        );
    }
    assert!(RegionMetrics::checked(1.0, cpu(0, 0), cpu(0, 0), cpu(0, 0), cpu(0, 0), 0).is_err());
}

#[test]
fn decreased_self_or_child_cpu_is_rejected() {
    for (before, after, child_before, child_after) in [
        (cpu(2, 0), cpu(1, 0), cpu(0, 0), cpu(0, 0)),
        (cpu(0, 2), cpu(0, 1), cpu(0, 0), cpu(0, 0)),
        (cpu(0, 0), cpu(0, 0), cpu(2, 0), cpu(1, 0)),
        (cpu(0, 0), cpu(0, 0), cpu(0, 2), cpu(0, 1)),
    ] {
        assert!(
            RegionMetrics::checked(1.0, before, after, child_before, child_after, 1024).is_err()
        );
    }
}

#[test]
fn unexpected_child_cpu_is_rejected_not_subtracted() {
    for after in [cpu(1, 0), cpu(0, 1)] {
        let error = RegionMetrics::checked(1.0, cpu(0, 0), cpu(10, 10), cpu(0, 0), after, 1024)
            .unwrap_err();
        assert!(error.contains("unexpected child CPU"));
    }
}

#[test]
fn elapsed_clock_direction_is_checked() {
    let start = Instant::now();
    let end = start + Duration::from_millis(250);
    assert_eq!(trial::checked_elapsed(start, end).unwrap(), 0.25);
    assert!(trial::checked_elapsed(end, start).is_err());
}

#[test]
fn thread_snapshot_requires_one_positive_counter() {
    assert_eq!(
        trial::parse_threads("Name:\ttrial\nThreads:\t3\nVmRSS:\t4 kB\n").unwrap(),
        3
    );
    for invalid in [
        "",
        "Threads: 0\n",
        "Threads: -1\n",
        "Threads: +1\n",
        "Threads: one\n",
        "Threads: 1 extra\n",
        "Threads: 1\nThreads: 2\n",
        "Threads: 18446744073709551616\n",
    ] {
        assert!(trial::parse_threads(invalid).is_err(), "{invalid:?}");
    }
}

#[test]
fn metadata_tables_are_exclusive_and_literal() {
    let root = TempRoot::new();
    let path = root.0.join("runtime.tsv");
    let rows = vec![("quoted".into(), " a\"b ".into())];
    trial::write_table(&path, &rows).unwrap();
    let before = fs::read(&path).unwrap();
    assert_eq!(before, b"key\tvalue\nquoted\t a\"b \n");
    assert!(trial::write_table(&path, &rows).is_err());
    assert_eq!(fs::read(path).unwrap(), before);
}

#[test]
fn metadata_separators_fail_before_file_creation() {
    let root = TempRoot::new();
    for (index, invalid) in ["bad\tfield", "bad\nrow", "bad\rrow"]
        .into_iter()
        .enumerate()
    {
        let path = root.0.join(format!("invalid-{index}.tsv"));
        assert!(trial::write_table(&path, &[("key".into(), invalid.into())]).is_err());
        assert!(!path.exists());
    }
}

#[test]
fn entry_point_requires_the_bounded_native_host() {
    assert_eq!(
        trial::require_measurement_host().is_ok(),
        cfg!(all(target_os = "linux", target_arch = "x86_64"))
    );
}

#[cfg(target_os = "linux")]
fn populate(root: &Path) {
    use flate2::{Compression, write::GzEncoder};
    use std::io::Write;
    let counts =
        "gene\tc_a\tc_obs\tc_b\nG_X\t100\t100\t100\nG_A\t-0\t3.75\t5.125\nG_B\t2.5\t4\t6.5\n";
    let rows = [
        (trial::PLAIN_COUNTS, counts),
        (
            trial::POSITIONS,
            "G_X\tchrX\t1\t2\nG_A\tchr1\t10\t20\nG_B\tchr2\t30\t40\n",
        ),
        (trial::ANNOTATIONS, "c_a\tr_a\nc_obs\to\nc_b\tr_b\n"),
        (
            "shipped-bundle/full/01.tsv",
            "gene\tc_a\tc_obs\tc_b\nG_A\t-0\t3.75\t5.125\nG_B\t2.5\t4\t6.5\n",
        ),
        (
            "shipped-bundle/full/01.genes.tsv",
            "gene\tchr\tstart\tstop\nG_A\tchr1\t10\t20\nG_B\tchr2\t30\t40\n",
        ),
        (
            "shipped-bundle/full/01.cells.tsv",
            "cell\tgroup\trole\nc_a\tr_a\treference\nc_obs\to\tobservation\nc_b\tr_b\treference\n",
        ),
        (
            trial::EXPECTED_MAPS,
            "role\tgroup\tindex\tcell\nreference\tr_b\t3\tc_b\nreference\tr_a\t1\tc_a\nobservation\to\t2\tc_obs\n",
        ),
    ];
    for (relative, text) in rows {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    let gzip = root.join(trial::GZIP_COUNTS);
    fs::create_dir_all(gzip.parent().unwrap()).unwrap();
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(counts.as_bytes()).unwrap();
    fs::write(gzip, encoder.finish().unwrap()).unwrap();
}

#[cfg(target_os = "linux")]
fn table(path: &Path) -> BTreeMap<String, String> {
    let text = fs::read_to_string(path).unwrap();
    let mut lines = text.lines();
    assert_eq!(lines.next(), Some("key\tvalue"));
    let mut fields = BTreeMap::new();
    for line in lines {
        let (key, value) = line.split_once('\t').unwrap();
        assert!(fields.insert(key.to_owned(), value.to_owned()).is_none());
    }
    fields
}

#[cfg(target_os = "linux")]
#[test]
fn real_plain_and_gzip_trials_validate_both_factory_returns() {
    for case in [InputCase::Plain, InputCase::Gzip] {
        let root = TempRoot::new();
        populate(&root.0);
        let args = Args {
            artifact_root: root.0.clone(),
            case,
            output: root.0.join("trial"),
        };
        let argv = vec![
            OsString::from("contract-trial"),
            args.artifact_root.clone().into_os_string(),
            case.name().into(),
            args.output.clone().into_os_string(),
        ];
        trial::run_trial(&args, &argv).unwrap();
        for file in ["expression.tsv", "genes.tsv", "cells.tsv", "maps.tsv"] {
            assert_eq!(
                fs::read(args.output.join("warm").join(file)).unwrap(),
                fs::read(args.output.join("measured").join(file)).unwrap()
            );
        }
        assert_eq!(
            fs::read(args.output.join("warm/maps.tsv")).unwrap(),
            fs::read(root.0.join(trial::EXPECTED_MAPS)).unwrap()
        );
        let metrics = table(&args.output.join("metrics.tsv"));
        assert_eq!(metrics.len(), 12);
        assert_eq!(metrics["input_case"], case.name());
        assert_eq!(metrics["measurement"], "raw_creation");
        assert!(metrics["region_wall_seconds"].parse::<f64>().unwrap() > 0.0);
        assert!(metrics["baseline_rss_bytes"].parse::<u64>().unwrap() > 0);
        assert!(metrics["returned_rss_bytes"].parse::<u64>().unwrap() > 0);
        assert_eq!(
            metrics["region_child_user_seconds"].parse::<f64>().unwrap(),
            0.0
        );
        assert_eq!(
            metrics["region_child_system_seconds"]
                .parse::<f64>()
                .unwrap(),
            0.0
        );
        let runtime = table(&args.output.join("runtime.tsv"));
        assert_eq!(runtime["reference_group_1"], "r_b");
        assert_eq!(runtime["reference_group_2"], "r_a");
        assert_eq!(runtime["returned_genes"], "2");
        assert_eq!(runtime["returned_cells"], "3");
        assert_eq!(runtime["required_input_run_id"], "36707344893");
        assert_eq!(runtime["required_input_artifact_id"], "11092988090");
        assert!(runtime["observed_threads_before"].parse::<u64>().unwrap() > 0);
        assert!(runtime["observed_threads_returned"].parse::<u64>().unwrap() > 0);
        assert!(args.output.join("io.lifetime.txt").is_file());
    }
}

#[cfg(target_os = "linux")]
#[test]
fn warm_validation_failure_does_not_produce_a_sample() {
    let root = TempRoot::new();
    populate(&root.0);
    let path = root.0.join("shipped-bundle/full/01.tsv");
    fs::write(&path, fs::read_to_string(&path).unwrap().replace("-0", "0")).unwrap();
    let args = Args {
        artifact_root: root.0.clone(),
        case: InputCase::Plain,
        output: root.0.join("failed"),
    };
    let error = trial::run_trial(&args, &[]).unwrap_err();
    assert!(error.contains("warm validation"));
    assert!(!args.output.join("metrics.tsv").exists());
    assert!(!args.output.join("warm").exists());
    assert!(!args.output.join("measured").exists());
}

#[cfg(target_os = "linux")]
#[test]
fn factory_parse_failure_is_not_a_timing_sample() {
    let root = TempRoot::new();
    populate(&root.0);
    let path = root.0.join(trial::PLAIN_COUNTS);
    fs::write(
        &path,
        fs::read_to_string(&path).unwrap().replace("-0", "NaN"),
    )
    .unwrap();
    let args = Args {
        artifact_root: root.0.clone(),
        case: InputCase::Plain,
        output: root.0.join("failed"),
    };
    let error = trial::run_trial(&args, &[]).unwrap_err();
    assert!(error.contains("warm creation"));
    assert!(!args.output.join("metrics.tsv").exists());
    assert!(!args.output.join("measured").exists());
}

#[cfg(target_os = "linux")]
#[test]
fn existing_trial_directory_preserves_its_prior_bytes() {
    let root = TempRoot::new();
    populate(&root.0);
    let args = Args {
        artifact_root: root.0.clone(),
        case: InputCase::Plain,
        output: root.0.join("existing"),
    };
    fs::create_dir(&args.output).unwrap();
    fs::write(args.output.join("sentinel"), "keep").unwrap();
    assert!(trial::run_trial(&args, &[]).is_err());
    assert_eq!(
        fs::read_to_string(args.output.join("sentinel")).unwrap(),
        "keep"
    );
    assert_eq!(fs::read_dir(args.output).unwrap().count(), 1);
}
```

### Task 2: Reviewed private implementation

**Files:** Add benches/ingestion_trial/mod.rs and benches/cnv_ingestion_matched.rs, plus this Cargo registration after genuine RED:

```toml
[[bench]]
name = "cnv_ingestion_matched"
path = "benches/cnv_ingestion_matched.rs"
harness = false
test = false
required-features = ["infercnv-measurement"]
```

- [x] Add exactly the complete helper/main below; external standalone Rust1.91 format. Recheck all production/old support/lock hashes.
- [x] Freeze clean green-13 with identical red-8 test bytes and only helper/main/Cargo bench additions. Three sorted snapshot checksums/exact tar manifest required. Independent complete-source review.
- [ ] Push owned world snapshot/ledger updates; wait exact-head Control plane; dispatch full four-native with all three measurement flags and accepted shipped oracle.
- [ ] Dedicated trial target debug/release; compile new bench with cargo bench --locked --features infercnv-measurement --bench cnv_ingestion_matched --no-run. Preserve original logs. Exact expected collected trial counts are19 Linux/14 macOS from source inventory, not a prior test result.
- [ ] Preserve the55 creation support,11 old support,72 ordinary/external tests per target/profile, format/strict Clippy and old bench. Correct any observed defects via narrow tests, fresh immutable snapshots and original failures; no waiver.

Complete helper:

```rust
#[allow(
    dead_code,
    reason = "this trial reuses only checked counters and environment formatting"
)]
#[path = "../support/mod.rs"]
mod counters;
#[cfg(target_os = "linux")]
#[path = "../ingestion_support/mod.rs"]
mod creation;

use rsomics_sc::cnv::{CreationConfig, InputPaths};
use std::ffi::{OsStr, OsString};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

#[cfg(target_os = "linux")]
use nix::sys::resource::{UsageWho, getrusage};
#[cfg(target_os = "linux")]
use rsomics_sc::cnv::{CreatedInput, create_input};
#[cfg(target_os = "linux")]
use std::fs;

type Result<T> = std::result::Result<T, String>;

pub const PLAIN_COUNTS: &str = "shipped-bundle/inputs/canonical-full.tsv";
pub const GZIP_COUNTS: &str = "ingestion-witness/shipped_full/counts.tsv.gz";
pub const POSITIONS: &str = "shipped-bundle/inputs/gencode_downsampled.EXAMPLE_ONLY_DONT_REUSE.txt";
pub const ANNOTATIONS: &str = "shipped-bundle/inputs/oligodendroglioma_annotations_downsampled.txt";
pub const EXPECTED_FULL: &str = "shipped-bundle/full";
pub const EXPECTED_MAPS: &str = "ingestion-witness/shipped_full/plain/maps.tsv";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputCase {
    Plain,
    Gzip,
}

impl InputCase {
    fn parse(value: &OsStr) -> Result<Self> {
        if value == "plain" {
            Ok(Self::Plain)
        } else if value == "gzip" {
            Ok(Self::Gzip)
        } else {
            Err("input case must be plain or gzip".into())
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::Gzip => "gzip",
        }
    }

    pub fn check_magic(self, path: &Path) -> Result<()> {
        let mut file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
        let mut magic = [0_u8; 2];
        file.read_exact(&mut magic)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        if (magic == [0x1f, 0x8b]) != (self == Self::Gzip) {
            return Err(format!(
                "{}: file magic disagrees with input case",
                path.display()
            ));
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Args {
    pub artifact_root: PathBuf,
    pub case: InputCase,
    pub output: PathBuf,
}

impl Args {
    pub fn parse(values: &[OsString]) -> Result<Self> {
        if values.len() != 3 || values[0].is_empty() || values[2].is_empty() {
            return Err(
                "usage: cnv_ingestion_matched ARTIFACT_ROOT plain|gzip NEW_OUTPUT_DIR".into(),
            );
        }
        Ok(Self {
            artifact_root: PathBuf::from(&values[0]),
            case: InputCase::parse(&values[1])?,
            output: PathBuf::from(&values[2]),
        })
    }
}

pub struct Inputs {
    pub counts: PathBuf,
    pub plain_counts: PathBuf,
    pub positions: PathBuf,
    pub annotations: PathBuf,
    pub expected_full: PathBuf,
    pub expected_maps: PathBuf,
}

impl Inputs {
    pub fn new(root: &Path, case: InputCase) -> Self {
        Self {
            counts: root.join(match case {
                InputCase::Plain => PLAIN_COUNTS,
                InputCase::Gzip => GZIP_COUNTS,
            }),
            plain_counts: root.join(PLAIN_COUNTS),
            positions: root.join(POSITIONS),
            annotations: root.join(ANNOTATIONS),
            expected_full: root.join(EXPECTED_FULL),
            expected_maps: root.join(EXPECTED_MAPS),
        }
    }

    pub fn paths(&self) -> InputPaths<'_> {
        InputPaths {
            counts: &self.counts,
            positions: &self.positions,
            annotations: &self.annotations,
        }
    }
}

pub fn creation_config(reference_names: Vec<String>) -> CreationConfig {
    CreationConfig {
        reference_groups: reference_names,
        excluded_chromosomes: vec!["chrX".into(), "chrY".into(), "chrM".into()],
        min_counts_per_cell: 1.0,
        max_counts_per_cell: f64::INFINITY,
        max_numeric_bytes: 256 * 1024 * 1024,
        max_record_bytes: 1024 * 1024,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CpuSample {
    pub user_micros: u64,
    pub system_micros: u64,
}

#[derive(Debug)]
pub struct RegionMetrics {
    pub wall_seconds: f64,
    pub user_seconds: f64,
    pub system_seconds: f64,
    pub child_user_seconds: f64,
    pub child_system_seconds: f64,
    pub returned_rss_bytes: u64,
}

impl RegionMetrics {
    pub fn checked(
        wall_seconds: f64,
        before: CpuSample,
        after: CpuSample,
        child_before: CpuSample,
        child_after: CpuSample,
        returned_rss_bytes: u64,
    ) -> Result<Self> {
        if !wall_seconds.is_finite() || wall_seconds <= 0.0 || returned_rss_bytes == 0 {
            return Err("region requires positive finite wall time and positive live RSS".into());
        }
        let delta = |before, after| {
            counters::checked_counter_delta(before, after).map(|micros| micros as f64 / 1_000_000.0)
        };
        let user_seconds = delta(before.user_micros, after.user_micros)?;
        let system_seconds = delta(before.system_micros, after.system_micros)?;
        let child_user_seconds = delta(child_before.user_micros, child_after.user_micros)?;
        let child_system_seconds = delta(child_before.system_micros, child_after.system_micros)?;
        if child_user_seconds != 0.0 || child_system_seconds != 0.0 {
            return Err("unexpected child CPU work during creation".into());
        }
        Ok(Self {
            wall_seconds,
            user_seconds,
            system_seconds,
            child_user_seconds,
            child_system_seconds,
            returned_rss_bytes,
        })
    }
}

pub fn checked_elapsed(start: Instant, end: Instant) -> Result<f64> {
    end.checked_duration_since(start)
        .map(|duration| duration.as_secs_f64())
        .ok_or_else(|| "wall clock reversed".into())
}

pub fn parse_threads(text: &str) -> Result<u64> {
    let mut count = None;
    for line in text.lines() {
        let Some(value) = line.strip_prefix("Threads:") else {
            continue;
        };
        if count.is_some() {
            return Err("duplicate process thread count".into());
        }
        let mut fields = value.split_whitespace();
        let token = fields
            .next()
            .ok_or_else(|| "missing process thread count".to_owned())?;
        if token.is_empty()
            || !token.bytes().all(|byte| byte.is_ascii_digit())
            || fields.next().is_some()
        {
            return Err("invalid process thread count".into());
        }
        let value = token
            .parse::<u64>()
            .map_err(|error| format!("process thread count: {error}"))?;
        if value == 0 {
            return Err("process thread count must be positive".into());
        }
        count = Some(value);
    }
    count.ok_or_else(|| "missing process thread count".into())
}

pub fn write_table(path: &Path, rows: &[(String, String)]) -> Result<()> {
    for (key, value) in rows {
        for field in [key, value] {
            counters::format_env_value(Some(OsString::from(field.as_str())))?;
        }
    }
    let result = (|| {
        let file = OpenOptions::new().write(true).create_new(true).open(path)?;
        let mut writer = BufWriter::new(file);
        writeln!(writer, "key\tvalue")?;
        for (key, value) in rows {
            writeln!(writer, "{key}\t{value}")?;
        }
        writer.flush()
    })();
    result.map_err(|error| format!("{}: {error}", path.display()))
}

pub fn require_measurement_host() -> Result<()> {
    if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Ok(())
    } else {
        Err("cnv_ingestion_matched requires native Linux x86_64".into())
    }
}

pub fn run_env() -> Result<()> {
    require_measurement_host()?;
    let argv: Vec<_> = std::env::args_os().collect();
    let args = Args::parse(&argv[1..])?;
    run_trial(&args, &argv)
}

#[cfg(not(target_os = "linux"))]
pub fn run_trial(_args: &Args, _argv: &[OsString]) -> Result<()> {
    Err("trial counters require Linux procfs and getrusage".into())
}

#[cfg(target_os = "linux")]
fn read_current_rss() -> Result<u64> {
    let path = "/proc/self/smaps_rollup";
    let text = fs::read_to_string(path).map_err(|error| format!("{path}: {error}"))?;
    counters::parse_rss_bytes(&text).map_err(|error| format!("{path}: {error}"))
}

#[cfg(target_os = "linux")]
fn read_process_threads() -> Result<u64> {
    let path = "/proc/self/status";
    let text = fs::read_to_string(path).map_err(|error| format!("{path}: {error}"))?;
    parse_threads(&text).map_err(|error| format!("{path}: {error}"))
}

#[cfg(target_os = "linux")]
fn process_cpu(who: UsageWho) -> Result<CpuSample> {
    let usage = getrusage(who).map_err(|error| format!("getrusage: {error}"))?;
    let user = usage.user_time();
    let system = usage.system_time();
    Ok(CpuSample {
        user_micros: counters::checked_cpu_micros(user.tv_sec(), user.tv_usec())?,
        system_micros: counters::checked_cpu_micros(system.tv_sec(), system.tv_usec())?,
    })
}

#[cfg(target_os = "linux")]
fn measure_creation(
    paths: InputPaths<'_>,
    config: &CreationConfig,
) -> Result<(CreatedInput, RegionMetrics)> {
    let child_before = process_cpu(UsageWho::RUSAGE_CHILDREN)?;
    let self_before = process_cpu(UsageWho::RUSAGE_SELF)?;
    let start = Instant::now();
    let created =
        create_input(paths, config).map_err(|error| format!("measured creation: {error}"))?;
    let end = Instant::now();
    let self_after = process_cpu(UsageWho::RUSAGE_SELF)?;
    let child_after = process_cpu(UsageWho::RUSAGE_CHILDREN)?;
    let returned_rss_bytes = read_current_rss()?;
    let metrics = RegionMetrics::checked(
        checked_elapsed(start, end)?,
        self_before,
        self_after,
        child_before,
        child_after,
        returned_rss_bytes,
    )?;
    Ok((created, metrics))
}

#[cfg(target_os = "linux")]
pub fn run_trial(args: &Args, argv: &[OsString]) -> Result<()> {
    let preparation_start = Instant::now();
    let artifact_root = fs::canonicalize(&args.artifact_root)
        .map_err(|error| format!("{}: {error}", args.artifact_root.display()))?;
    let output =
        std::path::absolute(&args.output).map_err(|error| format!("output path: {error}"))?;
    let inputs = Inputs::new(&artifact_root, args.case);
    args.case.check_magic(&inputs.counts)?;
    let expected = creation::load_expected(&inputs.expected_full, &inputs.expected_maps)?;
    let config = creation_config(expected.reference_names());
    let path_value = |path: &Path| counters::format_env_value(Some(path.as_os_str().to_owned()));
    let file_bytes = |path: &Path| {
        fs::metadata(path)
            .map(|metadata| metadata.len())
            .map_err(|error| format!("{}: {error}", path.display()))
    };
    let mut runtime = vec![
        ("schema_version".into(), "1".into()),
        ("measurement".into(), "raw_creation".into()),
        ("implementation".into(), "rust".into()),
        ("factory".into(), "rsomics_sc::cnv::create_input".into()),
        ("product_version".into(), env!("CARGO_PKG_VERSION").into()),
        ("operating_system".into(), std::env::consts::OS.into()),
        ("architecture".into(), std::env::consts::ARCH.into()),
        (
            "build_assertions".into(),
            cfg!(debug_assertions).to_string(),
        ),
        ("input_case".into(), args.case.name().into()),
        (
            "required_input_receipt".into(),
            "infercnv-ingestion-measurement-input-2026-09-30.json".into(),
        ),
        ("required_input_run_id".into(), "36707344893".into()),
        ("required_input_artifact_id".into(), "11092988090".into()),
        (
            "required_input_artifact_sha256".into(),
            "2481188b6f3d058d154bf16df9297a9eca261c6df84280e7d16569c92da28bd8".into(),
        ),
        (
            "required_input_receipt_sha256".into(),
            "3bee1dc582a321f16cb173c757e955dc620a911a891f4051f8bae68dc856dc0b".into(),
        ),
        ("artifact_root".into(), path_value(&artifact_root)?),
        ("counts_path".into(), path_value(&inputs.counts)?),
        ("positions_path".into(), path_value(&inputs.positions)?),
        ("annotations_path".into(), path_value(&inputs.annotations)?),
        (
            "expected_full_path".into(),
            path_value(&inputs.expected_full)?,
        ),
        (
            "expected_maps_path".into(),
            path_value(&inputs.expected_maps)?,
        ),
        (
            "input_selected_file_bytes".into(),
            file_bytes(&inputs.counts)?.to_string(),
        ),
        (
            "input_decoded_count_bytes".into(),
            file_bytes(&inputs.plain_counts)?.to_string(),
        ),
        (
            "decoded_bytes_binding".into(),
            "canonical plain file length; gzip equality checked by external controller".into(),
        ),
        (
            "input_authentication".into(),
            "external controller must verify receipt paths and hashes before and after all trials"
                .into(),
        ),
        (
            "max_numeric_bytes".into(),
            config.max_numeric_bytes.to_string(),
        ),
        (
            "max_record_bytes".into(),
            config.max_record_bytes.to_string(),
        ),
        ("min_counts_per_cell".into(), "1".into()),
        ("max_counts_per_cell".into(), "Inf".into()),
        (
            "count_representation".into(),
            "native f64 column-major".into(),
        ),
        (
            "numeric_parity_scope".into(),
            "frozen canonical receipt; not arbitrary R-decimal parsing".into(),
        ),
        ("factory_warmups_per_process".into(), "1".into()),
        ("measured_factory_calls_per_process".into(), "1".into()),
        (
            "warm_regime".into(),
            "filesystem and allocation warm; no controlled cold cache".into(),
        ),
        (
            "reclamation_before_baseline".into(),
            "drop warm CreatedInput and ExpectedCreation; no allocator trimming".into(),
        ),
        (
            "retained_preparation".into(),
            "paths/config/runtime metadata and possible allocator-retained pages".into(),
        ),
        (
            "cpu_counter_scope".into(),
            "getrusage self and reaped children; bracketing calls outside wall timer".into(),
        ),
        (
            "rss_scope".into(),
            "baseline and live return snapshots; neither is a transient factory peak".into(),
        ),
        (
            "whole_process_metrics".into(),
            "external monitor; includes warmup validation and exports".into(),
        ),
        (
            "thread_observation".into(),
            "procfs snapshots; no claim of continuous single-thread execution".into(),
        ),
        (
            "io_snapshot_scope".into(),
            "whole-process lifetime after both validated exports before metadata writes".into(),
        ),
        (
            "io_rchar_wchar".into(),
            "bytes from read/write-like syscalls including instrumentation".into(),
        ),
        (
            "io_syscr_syscw".into(),
            "read/write-like syscall counts".into(),
        ),
        (
            "io_read_write_bytes".into(),
            "Linux storage-layer accounting; cached physical reads may be zero".into(),
        ),
        (
            "io_cancelled_write_bytes".into(),
            "Linux accounting of dirty writes cancelled before storage".into(),
        ),
        ("pid".into(), std::process::id().to_string()),
        (
            "executable_path".into(),
            path_value(
                &std::env::current_exe().map_err(|error| format!("current executable: {error}"))?,
            )?,
        ),
    ];
    for (index, argument) in argv.iter().enumerate() {
        runtime.push((
            format!("argv_{index}"),
            counters::format_env_value(Some(argument.clone()))?,
        ));
    }
    for (index, reference) in config.reference_groups.iter().enumerate() {
        runtime.push((format!("reference_group_{}", index + 1), reference.clone()));
    }
    for (index, chromosome) in config.excluded_chromosomes.iter().enumerate() {
        runtime.push((
            format!("excluded_chromosome_{}", index + 1),
            chromosome.clone(),
        ));
    }
    for key in [
        "OMP_NUM_THREADS",
        "OPENBLAS_NUM_THREADS",
        "MKL_NUM_THREADS",
        "BLIS_NUM_THREADS",
        "NUMEXPR_NUM_THREADS",
        "RAYON_NUM_THREADS",
    ] {
        runtime.push((
            key.into(),
            counters::format_env_value(std::env::var_os(key))?,
        ));
    }

    fs::create_dir(&output).map_err(|error| format!("{}: {error}", output.display()))?;
    let warm =
        create_input(inputs.paths(), &config).map_err(|error| format!("warm creation: {error}"))?;
    creation::validate_created(&warm, &expected)
        .map_err(|error| format!("warm validation: {error}"))?;
    creation::write_creation(&output.join("warm"), &warm)?;
    drop(warm);
    drop(expected);
    let threads_before = read_process_threads()?;
    let baseline_rss_bytes = read_current_rss()?;
    let preparation_end = Instant::now();
    let (measured, metrics) = measure_creation(inputs.paths(), &config)?;
    let threads_returned = read_process_threads()?;

    let expected = creation::load_expected(&inputs.expected_full, &inputs.expected_maps)?;
    creation::validate_created(&measured, &expected)
        .map_err(|error| format!("measured validation: {error}"))?;
    creation::write_creation(&output.join("measured"), &measured)?;
    runtime.push((
        "returned_genes".into(),
        measured.prepared_counts().state().gene_count().to_string(),
    ));
    runtime.push((
        "returned_cells".into(),
        measured.prepared_counts().state().cell_count().to_string(),
    ));
    runtime.push(("observed_threads_before".into(), threads_before.to_string()));
    runtime.push((
        "observed_threads_returned".into(),
        threads_returned.to_string(),
    ));

    let io_path = "/proc/self/io";
    let io = fs::read(io_path).map_err(|error| format!("{io_path}: {error}"))?;
    let destination = output.join("io.lifetime.txt");
    let mut io_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&destination)
        .map_err(|error| format!("{}: {error}", destination.display()))?;
    io_file
        .write_all(&io)
        .and_then(|()| io_file.flush())
        .map_err(|error| format!("{}: {error}", destination.display()))?;
    write_table(&output.join("runtime.tsv"), &runtime)?;
    write_table(
        &output.join("metrics.tsv"),
        &[
            ("schema_version".into(), "1".into()),
            ("measurement".into(), "raw_creation".into()),
            ("implementation".into(), "rust".into()),
            ("input_case".into(), args.case.name().into()),
            (
                "preparation_wall_seconds".into(),
                format!(
                    "{:.17e}",
                    checked_elapsed(preparation_start, preparation_end)?
                ),
            ),
            (
                "region_wall_seconds".into(),
                format!("{:.17e}", metrics.wall_seconds),
            ),
            (
                "region_user_seconds".into(),
                format!("{:.17e}", metrics.user_seconds),
            ),
            (
                "region_system_seconds".into(),
                format!("{:.17e}", metrics.system_seconds),
            ),
            (
                "region_child_user_seconds".into(),
                format!("{:.17e}", metrics.child_user_seconds),
            ),
            (
                "region_child_system_seconds".into(),
                format!("{:.17e}", metrics.child_system_seconds),
            ),
            ("baseline_rss_bytes".into(), baseline_rss_bytes.to_string()),
            (
                "returned_rss_bytes".into(),
                metrics.returned_rss_bytes.to_string(),
            ),
        ],
    )
}
```

Complete bench main:

```rust
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[path = "ingestion_trial/mod.rs"]
mod trial;

fn main() {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    if let Err(error) = trial::run_env() {
        eprintln!("cnv_ingestion_matched: {error}");
        std::process::exit(1);
    }
    #[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
    {
        eprintln!("cnv_ingestion_matched requires native Linux x86_64");
        std::process::exit(1);
    }
}
```

### Task 3: Original artifact acceptance and product commit

- [ ] Root authenticates exact run/head/attempt/native jobs, original artifact API digest/size/ZIP CRC/safe names and source/guard bytes. Archive exact inventory and all3 checksums, before/after source, native Rust1.91, lock and production unchanged. Inspect actual test summaries and new bench artifacts.
- [ ] Only after four-native success/audit, commit owning product's4 new/changed paths (Cargo.toml, new trial test/helper/bench) with test(sc): measure actual raw creation boundaries. Do not include other files or force-stage inherited work.
- [ ] Record exact accepted gate in tracked world ledger/receipt and push with exact-head control CI.

## Separate subsequent acceptance

No actual canonical fresh processes or speed is accepted here. The next R trial/package/paired-driver plan must authenticate code/import chains BEFORE process launch and after; validate64 outputs from32 fresh processes, omitted/error/mixed-provenance regression tests, all counters and trial inventories. Preserve GNU time whole-process totals separately; per-case strict max7Rust<min7R is preselected. No peak factory-memory claim, cold-cache/continuous-thread guarantee, general R numeric parity, downstream clustering, whole inferCNV or publication follows this private trial.
