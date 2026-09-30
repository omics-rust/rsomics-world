# Private exact Rust creation support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Bounded read-only proposal/review agents may work in parallel; root owns integration and remote gates.

**Goal:** Provide private exact creation comparison/export support with contract tests, without modifying accepted production code or claiming ingestion performance.

**Architecture:** Reuse the checked product state and trusted single-stage fixture loader. Add an ordered role/map boundary and a non-destructive four-file exporter, enabled only by the existing private measurement feature.

**Tech Stack:** Rust 1.91, existing rsomics-sc/flate2 dependencies, four-native hosted CI and Python controller checks.

**Spec:** [Private creation support design](2026-09-30-infercnv-private-creation-support-design.md), within [raw-ingestion measurement](2026-09-30-infercnv-raw-ingestion-measurement-design.md).

## Global Constraints

- Owner: `/Volumes/KIOXIA/Documents/omics-rust/rsomics-sc`, baseline `e62fc960aeeb9dc543298885213cda6a57196fe9`.
- Change only Cargo.toml registration, new `tests/cnv_ingestion_measurement_contract.rs` and new `benches/ingestion_support/mod.rs` in that repository.
- Existing production, fixtures/tests, old benchmark/support and Cargo.lock remain byte-identical. No dependencies, public APIs, new crates, binary or publication.
- Expected counts compare `f64::to_bits()` including negative zero; identities/coordinates and ordered maps compare exactly.
- Reuse PreparedCounts invariants; do not duplicate validation of private CreatedInput fields. Preserve literal quotes/spaces/Unicode, reject TSV separators before output creation.
- Existing output is never overwritten/deleted. Errors propagate with partial evidence preserved; the callback-error test does not prove an operating-system flush fault.
- Work directly on main per the user's manual; root commits only owned changes after review/gates. No PR or Co-Authored-By.
- Boot occupancy is 88.68%, so no local Cargo or dependency download. Standalone formatter and controller TMPDIR use external disks; actual Cargo uses hosted runner temporary paths.
- Candidate snapshot RED=`red-7`, GREEN=`green-11`; selectors default false. Native test targets are Linux/macOS x86_64/aarch64, debug and release.

## Review Focus

- Expected role/map metadata omitted by the old stage loader must still bind every actual cell exactly once; explicit role, coverage and identity mutations exercise this.
- Interleaved group members and requested reference order cannot become sorted membership sets; ordered-map and changed-request tests pin both.
- Negative zero and one-ULP mutations cannot become tolerance-based equivalence; bit tests and exporter reload prove this.
- Literal names must not become CSV-decoded aliases or malformed records; quotes/Unicode roundtrip and separator preflight tests exercise this.
- Existing or partially failed output must not be silently accepted or overwritten; directory collision/preservation and callback-error tests cover known branches, with real OS flush failure left to code-path review.

## Pre-flight interface review

| Tasks | Shared interface | Check |
| --- | --- | --- |
| 1 → 2 | `ExpectedCreation`, `load_expected`, `validate_created`, `write_creation` | Test module path and exact signatures match the complete implementation below. |
| 1 → 3 | `ingestion_measurement` plus `measurement` | Default-false selector requires measurement and invokes dedicated target in both profiles; older paths remain unchanged. |
| 2 → 3 | Four output files and production-byte contract | Existing source/lock hashes remain checked; no timing executable or performance claim is created. |

Self-review: all spec requirements have a code/test/gate below. PreparedCounts reuse removes duplicate numeric/model rules. Public-foundation promotion and actual paired trials are outside this independently testable component. Fresh spec reviewer checked actual constructors/loaders/exporters and found no load-bearing gap. Product README's historical raw-input description needs a separate later documentation concern; it is not silently rewritten inside this frozen gate.

### Task 1: Test-first contract and opt-in native selector

**Files:** create `tests/cnv_ingestion_measurement_contract.rs`; modify owning Cargo.toml; modify world `.github/workflows/sc-cnv-core-candidate.yml`; freeze world `.autopilot/snapshots/sc-cnv-core-2026-09-26/red-7/`.

**Interfaces:** consumes actual `create_input`, `CreationConfig`, `CreatedInput`; produces the dedicated `cnv_ingestion_measurement_contract` test target and exact four-function private support contract. The absent helper produces the intended RED.

- [x] **Step 1: Add the complete contract test below, before the helper exists.**

```rust
#[path = "../benches/ingestion_support/mod.rs"]
mod ingestion;

use flate2::{Compression, write::GzEncoder};
use ingestion::{ExpectedCreation, load_expected, validate_created, write_creation};
use rsomics_sc::cnv::{CreatedInput, CreationConfig, InputPaths, create_input};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct TempRoot(PathBuf);

impl TempRoot {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "rsomics-ingestion-contract-{}-{}",
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

struct Fixture {
    root: TempRoot,
    genes: [&'static str; 2],
    chromosomes: [&'static str; 2],
    cells: Vec<&'static str>,
    groups: Vec<&'static str>,
}

impl Fixture {
    fn new(extra_cell: bool, quoted: bool) -> Self {
        let mut cells = if quoted {
            vec![" c\"a ", "细胞\"obs", "c_b"]
        } else {
            vec!["c_a", "c_obs", "c_b"]
        };
        let mut groups = if quoted {
            vec![" r\"a ", " o\"q ", "r_b"]
        } else {
            vec!["r_a", "o", "r_b"]
        };
        if extra_cell {
            cells.push("c_a2");
            groups.push(groups[0]);
        }
        let fixture = Self {
            root: TempRoot::new(),
            genes: if quoted {
                [" G\"A ", "G_B"]
            } else {
                ["G_A", "G_B"]
            },
            chromosomes: if quoted {
                [" ch\"r1 ", "chr2"]
            } else {
                ["chr1", "chr2"]
            },
            cells,
            groups,
        };
        let expression = fixture.expression();
        fs::write(fixture.path("counts.tsv"), expression).unwrap();
        fs::write(
            fixture.path("positions.tsv"),
            format!(
                "{}\t{}\t10\t20\n{}\t{}\t30\t40\n",
                fixture.genes[0], fixture.chromosomes[0], fixture.genes[1], fixture.chromosomes[1]
            ),
        )
        .unwrap();
        let annotations: String = fixture
            .cells
            .iter()
            .zip(&fixture.groups)
            .map(|(cell, group)| format!("{cell}\t{group}\n"))
            .collect();
        fs::write(fixture.path("annotations.tsv"), annotations).unwrap();
        fixture.write_expected(&fixture.references());
        fixture
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.0.join(name)
    }

    fn references(&self) -> Vec<String> {
        vec!["r_b".into(), self.groups[0].into()]
    }

    fn expression(&self) -> String {
        let values = [["-0", "2.5"], ["3.75", "4"], ["5.125", "6.5"], ["7", "8"]];
        let mut text = format!("gene\t{}\n", self.cells.join("\t"));
        for (row, gene) in self.genes.iter().enumerate() {
            text.push_str(gene);
            for column in values.iter().take(self.cells.len()) {
                text.push('\t');
                text.push_str(column[row]);
            }
            text.push('\n');
        }
        text
    }

    fn write_expected(&self, references: &[String]) {
        fs::write(self.path("01.tsv"), self.expression()).unwrap();
        fs::write(
            self.path("01.genes.tsv"),
            format!(
                "gene\tchr\tstart\tstop\n{}\t{}\t10\t20\n{}\t{}\t30\t40\n",
                self.genes[0], self.chromosomes[0], self.genes[1], self.chromosomes[1]
            ),
        )
        .unwrap();

        let is_reference = |group: &str| references.iter().any(|name| name == group);
        let mut cells = "cell\tgroup\trole\n".to_owned();
        for (cell, group) in self.cells.iter().zip(&self.groups) {
            let role = if is_reference(group) {
                "reference"
            } else {
                "observation"
            };
            cells.push_str(&format!("{cell}\t{group}\t{role}\n"));
        }
        fs::write(self.path("01.cells.tsv"), cells).unwrap();

        let mut observations: Vec<&str> = self
            .groups
            .iter()
            .copied()
            .filter(|group| !is_reference(group))
            .collect();
        observations.sort_unstable_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        observations.dedup();
        let mut maps = "role\tgroup\tindex\tcell\n".to_owned();
        for (role, names) in [
            (
                "reference",
                references.iter().map(String::as_str).collect::<Vec<_>>(),
            ),
            ("observation", observations),
        ] {
            for name in names {
                for (index, (&cell, &group)) in self.cells.iter().zip(&self.groups).enumerate() {
                    if group == name {
                        maps.push_str(&format!("{role}\t{name}\t{}\t{cell}\n", index + 1));
                    }
                }
            }
        }
        fs::write(self.path("expected.maps.tsv"), maps).unwrap();
    }

    fn expected(&self) -> Result<ExpectedCreation, String> {
        load_expected(&self.root.0, &self.path("expected.maps.tsv"))
    }

    fn create(&self, references: &[String]) -> CreatedInput {
        let counts = self.path("counts.tsv");
        let positions = self.path("positions.tsv");
        let annotations = self.path("annotations.tsv");
        create_input(
            InputPaths {
                counts: &counts,
                positions: &positions,
                annotations: &annotations,
            },
            &CreationConfig {
                reference_groups: references.to_vec(),
                excluded_chromosomes: Vec::new(),
                min_counts_per_cell: 1.0,
                max_counts_per_cell: f64::INFINITY,
                max_numeric_bytes: 1 << 20,
                max_record_bytes: 1 << 16,
            },
        )
        .unwrap()
    }

    fn replace(&self, file: &str, from: &str, to: &str) {
        let path = self.path(file);
        let text = fs::read_to_string(&path).unwrap();
        assert!(
            text.contains(from),
            "{file}: missing replacement target {from:?}"
        );
        fs::write(path, text.replace(from, to)).unwrap();
    }

    fn assert_roundtrip(&self, references: &[String]) {
        let created = self.create(references);
        validate_created(&created, &self.expected().unwrap()).unwrap();
        let output = self.path("export");
        write_creation(&output, &created).unwrap();

        let mut names: Vec<_> = fs::read_dir(&output)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect();
        names.sort();
        assert_eq!(
            names,
            ["cells.tsv", "expression.tsv", "genes.tsv", "maps.tsv"]
        );
        assert_eq!(
            fs::read(output.join("cells.tsv")).unwrap(),
            fs::read(self.path("01.cells.tsv")).unwrap()
        );
        assert_eq!(
            fs::read(output.join("genes.tsv")).unwrap(),
            fs::read(self.path("01.genes.tsv")).unwrap()
        );
        assert_eq!(
            fs::read(output.join("maps.tsv")).unwrap(),
            fs::read(self.path("expected.maps.tsv")).unwrap()
        );

        let reload = self.path("reload");
        fs::create_dir(&reload).unwrap();
        for (from, to) in [
            ("expression.tsv", "01.tsv"),
            ("genes.tsv", "01.genes.tsv"),
            ("cells.tsv", "01.cells.tsv"),
        ] {
            fs::copy(output.join(from), reload.join(to)).unwrap();
        }
        let expected = load_expected(&reload, &output.join("maps.tsv")).unwrap();
        validate_created(&created, &expected).unwrap();
    }
}

#[test]
fn exact_column_major_values_and_reference_request_order() {
    let fixture = Fixture::new(false, false);
    let created = fixture.create(&fixture.references());
    let actual: Vec<_> = created
        .prepared_counts()
        .state()
        .values()
        .iter()
        .map(|value| value.to_bits())
        .collect();
    let wanted: Vec<_> = [-0.0_f64, 2.5, 3.75, 4.0, 5.125, 6.5]
        .into_iter()
        .map(f64::to_bits)
        .collect();
    assert_eq!(actual, wanted);
    assert_eq!(created.reference_groups()[0].name(), "r_b");
    assert_eq!(created.reference_groups()[0].cell_indices(), [2]);
    assert_eq!(created.reference_groups()[1].name(), "r_a");
    assert_eq!(created.reference_groups()[1].cell_indices(), [0]);
    assert_eq!(
        fs::read_to_string(fixture.path("expected.maps.tsv")).unwrap(),
        "role\tgroup\tindex\tcell\nreference\tr_b\t3\tc_b\nreference\tr_a\t1\tc_a\nobservation\to\t2\tc_obs\n"
    );
    fixture.assert_roundtrip(&fixture.references());
}

#[test]
fn noncontiguous_group_members_preserve_cell_column_order() {
    let fixture = Fixture::new(true, false);
    let created = fixture.create(&fixture.references());
    assert_eq!(created.reference_groups()[1].cell_indices(), [0, 3]);
    assert_eq!(
        fs::read_to_string(fixture.path("expected.maps.tsv")).unwrap(),
        "role\tgroup\tindex\tcell\nreference\tr_b\t3\tc_b\nreference\tr_a\t1\tc_a\nreference\tr_a\t4\tc_a2\nobservation\to\t2\tc_obs\n"
    );
    fixture.assert_roundtrip(&fixture.references());
}

#[test]
fn negative_zero_is_not_positive_zero() {
    let fixture = Fixture::new(false, false);
    let created = fixture.create(&fixture.references());
    fixture.replace("01.tsv", "-0", "0");
    let expected = fixture.expected().unwrap();
    let error = validate_created(&created, &expected).unwrap_err();
    assert!(error.contains("bits differ at offset 0"));
}

#[test]
fn one_ulp_difference_is_rejected() {
    let fixture = Fixture::new(false, false);
    let created = fixture.create(&fixture.references());
    let adjacent = f64::from_bits(2.5_f64.to_bits() + 1);
    fixture.replace("01.tsv", "2.5", &format!("{adjacent:.17e}"));
    let error = validate_created(&created, &fixture.expected().unwrap()).unwrap_err();
    assert!(error.contains("bits differ at offset 1"));
}

#[test]
fn changed_actual_reference_request_is_rejected() {
    let fixture = Fixture::new(false, false);
    let created = fixture.create(&["r_a".into(), "r_b".into()]);
    let expected = fixture.expected().unwrap();
    assert_eq!(expected.reference_names(), ["r_b", "r_a"]);
    assert!(
        validate_created(&created, &expected)
            .unwrap_err()
            .contains("reference group")
    );
}

#[test]
fn valid_expected_reference_order_is_preserved_not_sorted() {
    let fixture = Fixture::new(false, false);
    let references = vec!["r_a".into(), "r_b".into()];
    fixture.write_expected(&references);
    assert_eq!(
        fixture.expected().unwrap().reference_names(),
        ["r_a", "r_b"]
    );
    fixture.assert_roundtrip(&references);
}

#[test]
fn literal_quotes_spaces_and_unicode_survive_export() {
    let fixture = Fixture::new(true, true);
    let created = fixture.create(&fixture.references());
    assert_eq!(created.prepared_counts().state().genes()[0].id, " G\"A ");
    assert_eq!(
        created.prepared_counts().state().genes()[0].chromosome,
        " ch\"r1 "
    );
    assert_eq!(created.prepared_counts().state().cells()[0].id, " c\"a ");
    assert_eq!(created.prepared_counts().state().cells()[1].id, "细胞\"obs");
    assert_eq!(created.reference_groups()[1].name(), " r\"a ");
    fixture.assert_roundtrip(&fixture.references());
}

#[test]
fn no_reference_groups_export_all_observation_roles() {
    let fixture = Fixture::new(true, false);
    let references = Vec::new();
    fixture.write_expected(&references);
    let created = fixture.create(&references);
    assert!(created.reference_groups().is_empty());
    assert_eq!(
        created
            .observation_groups()
            .iter()
            .map(|group| group.name())
            .collect::<Vec<_>>(),
        ["o", "r_a", "r_b"]
    );
    fixture.assert_roundtrip(&references);
}

#[test]
fn all_reference_groups_export_no_observation_roles() {
    let fixture = Fixture::new(true, false);
    let references = vec!["r_b".into(), "r_a".into(), "o".into()];
    fixture.write_expected(&references);
    let created = fixture.create(&references);
    assert!(created.observation_groups().is_empty());
    fixture.assert_roundtrip(&references);
}

#[test]
fn missing_expected_files_return_contextual_errors() {
    for name in [
        "01.tsv",
        "01.genes.tsv",
        "01.cells.tsv",
        "expected.maps.tsv",
    ] {
        let fixture = Fixture::new(false, false);
        fs::rename(fixture.path(name), fixture.path(&format!("{name}.saved"))).unwrap();
        let error = fixture.expected().err().unwrap();
        assert!(error.contains(&fixture.path(name).display().to_string()));
    }
}

#[test]
fn zero_depth_expected_cell_is_rejected() {
    let fixture = Fixture::new(false, false);
    fixture.replace("01.tsv", "2.5", "0");
    let error = fixture.expected().err().unwrap();
    assert!(error.contains("zero or nonfinite depth"));
}

#[test]
fn overflowing_depth_expected_cell_is_rejected() {
    let fixture = Fixture::new(false, false);
    fixture.replace("01.tsv", "-0", "1.7976931348623157e308");
    fixture.replace("01.tsv", "2.5", "1.7976931348623157e308");
    let error = fixture.expected().err().unwrap();
    assert!(error.contains("zero or nonfinite depth"));
}

#[test]
fn gzip_magic_input_roundtrips_exact_creation_exports() {
    let fixture = Fixture::new(true, false);
    let path = fixture.path("counts.tsv");
    let original = fs::read(&path).unwrap();
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&original).unwrap();
    let compressed = encoder.finish().unwrap();
    assert!(compressed.starts_with(&[0x1f, 0x8b]));
    fs::write(path, compressed).unwrap();
    fixture.assert_roundtrip(&fixture.references());
}

macro_rules! invalid_map {
    ($name:ident, $body:literal) => {
        #[test]
        fn $name() {
            let fixture = Fixture::new(true, false);
            fs::write(
                fixture.path("expected.maps.tsv"),
                concat!("role\tgroup\tindex\tcell\n", $body),
            )
            .unwrap();
            assert!(fixture.expected().is_err());
        }
    };
}

invalid_map!(
    unknown_map_role,
    "unknown\tr_b\t3\tc_b\nreference\tr_a\t1\tc_a\nreference\tr_a\t4\tc_a2\nobservation\to\t2\tc_obs\n"
);
invalid_map!(
    zero_map_index,
    "reference\tr_b\t0\tc_b\nreference\tr_a\t1\tc_a\nreference\tr_a\t4\tc_a2\nobservation\to\t2\tc_obs\n"
);
invalid_map!(
    out_of_range_map_index,
    "reference\tr_b\t5\tc_b\nreference\tr_a\t1\tc_a\nreference\tr_a\t4\tc_a2\nobservation\to\t2\tc_obs\n"
);
invalid_map!(
    nondecimal_map_index,
    "reference\tr_b\t+3\tc_b\nreference\tr_a\t1\tc_a\nreference\tr_a\t4\tc_a2\nobservation\to\t2\tc_obs\n"
);
invalid_map!(
    wrong_map_cell,
    "reference\tr_b\t3\tc_a\nreference\tr_a\t1\tc_a\nreference\tr_a\t4\tc_a2\nobservation\to\t2\tc_obs\n"
);
invalid_map!(
    wrong_map_group,
    "reference\tr_a\t3\tc_b\nreference\tr_a\t1\tc_a\nreference\tr_a\t4\tc_a2\nobservation\to\t2\tc_obs\n"
);
invalid_map!(
    wrong_map_role,
    "observation\tr_b\t3\tc_b\nreference\tr_a\t1\tc_a\nreference\tr_a\t4\tc_a2\nobservation\to\t2\tc_obs\n"
);
invalid_map!(
    duplicate_map_cell,
    "reference\tr_b\t3\tc_b\nreference\tr_a\t1\tc_a\nreference\tr_a\t1\tc_a\nreference\tr_a\t4\tc_a2\nobservation\to\t2\tc_obs\n"
);
invalid_map!(
    missing_map_cell,
    "reference\tr_b\t3\tc_b\nreference\tr_a\t1\tc_a\nreference\tr_a\t4\tc_a2\n"
);
invalid_map!(
    split_map_group,
    "reference\tr_a\t1\tc_a\nreference\tr_b\t3\tc_b\nreference\tr_a\t4\tc_a2\nobservation\to\t2\tc_obs\n"
);
invalid_map!(
    reference_after_observation,
    "reference\tr_b\t3\tc_b\nobservation\to\t2\tc_obs\nreference\tr_a\t1\tc_a\nreference\tr_a\t4\tc_a2\n"
);
invalid_map!(
    reversed_map_members,
    "reference\tr_b\t3\tc_b\nreference\tr_a\t4\tc_a2\nreference\tr_a\t1\tc_a\nobservation\to\t2\tc_obs\n"
);

#[test]
fn unordered_observation_groups_are_rejected() {
    let fixture = Fixture::new(true, false);
    fixture.write_expected(&[]);
    fs::write(
        fixture.path("expected.maps.tsv"),
        "role\tgroup\tindex\tcell\nobservation\tr_b\t3\tc_b\nobservation\tr_a\t1\tc_a\nobservation\tr_a\t4\tc_a2\nobservation\to\t2\tc_obs\n",
    )
    .unwrap();
    assert!(fixture.expected().is_err());
}

macro_rules! invalid_expected {
    ($name:ident, $file:literal, $from:literal, $to:literal) => {
        #[test]
        fn $name() {
            let fixture = Fixture::new(false, false);
            fixture.replace($file, $from, $to);
            assert!(fixture.expected().is_err());
        }
    };
}

invalid_expected!(
    cell_role_is_checked_independently,
    "01.cells.tsv",
    "c_a\tr_a\treference",
    "c_a\tr_a\tobservation"
);
invalid_expected!(
    unknown_cell_role,
    "01.cells.tsv",
    "c_a\tr_a\treference",
    "c_a\tr_a\tunknown"
);
invalid_expected!(
    empty_cell_group,
    "01.cells.tsv",
    "c_a\tr_a\treference",
    "c_a\t\treference"
);
invalid_expected!(
    literal_tab_in_identity_is_not_unquoted,
    "01.cells.tsv",
    "c_a\tr_a",
    "c\ta\tr_a"
);
invalid_expected!(
    literal_carriage_return_in_identity_is_rejected,
    "01.cells.tsv",
    "c_a\tr_a",
    "c\ra\tr_a"
);
invalid_expected!(
    reversed_coordinates,
    "01.genes.tsv",
    "chr1\t10\t20",
    "chr1\t21\t20"
);
invalid_expected!(
    zero_zero_coordinates,
    "01.genes.tsv",
    "chr1\t10\t20",
    "chr1\t0\t0"
);
invalid_expected!(
    nondecimal_coordinates,
    "01.genes.tsv",
    "chr1\t10\t20",
    "chr1\t+10\t20"
);
invalid_expected!(
    coordinate_overflow,
    "01.genes.tsv",
    "chr1\t10\t20",
    "chr1\t18446744073709551616\t20"
);
invalid_expected!(empty_chromosome, "01.genes.tsv", "G_A\tchr1", "G_A\t");
invalid_expected!(negative_expected_count, "01.tsv", "-0", "-1");
invalid_expected!(nonfinite_expected_count, "01.tsv", "-0", "NaN");
invalid_expected!(
    malformed_expression_width,
    "01.tsv",
    "G_A\t-0\t3.75\t5.125",
    "G_A\t-0\t3.75"
);

#[test]
fn changed_gene_identity_is_rejected() {
    let fixture = Fixture::new(false, false);
    let created = fixture.create(&fixture.references());
    fixture.replace("01.genes.tsv", "G_A", "G_X");
    fixture.replace("01.tsv", "G_A", "G_X");
    assert!(validate_created(&created, &fixture.expected().unwrap()).is_err());
}

#[test]
fn changed_chromosome_is_rejected() {
    let fixture = Fixture::new(false, false);
    let created = fixture.create(&fixture.references());
    fixture.replace("01.genes.tsv", "chr1", "chrX");
    assert!(validate_created(&created, &fixture.expected().unwrap()).is_err());
}

#[test]
fn changed_valid_coordinate_is_rejected() {
    let fixture = Fixture::new(false, false);
    let created = fixture.create(&fixture.references());
    fixture.replace("01.genes.tsv", "chr1\t10\t20", "chr1\t11\t20");
    assert!(validate_created(&created, &fixture.expected().unwrap()).is_err());
}

#[test]
fn changed_cell_identity_is_rejected() {
    let fixture = Fixture::new(false, false);
    let created = fixture.create(&fixture.references());
    for file in ["01.cells.tsv", "01.tsv", "expected.maps.tsv"] {
        fixture.replace(file, "c_a", "c_x");
    }
    assert!(validate_created(&created, &fixture.expected().unwrap()).is_err());
}

#[test]
fn changed_cell_group_is_rejected() {
    let fixture = Fixture::new(false, false);
    let created = fixture.create(&fixture.references());
    for file in ["01.cells.tsv", "expected.maps.tsv"] {
        fixture.replace(file, "r_a", "r_x");
    }
    assert!(validate_created(&created, &fixture.expected().unwrap()).is_err());
}

#[test]
fn changed_gene_row_order_is_rejected() {
    let fixture = Fixture::new(false, false);
    let created = fixture.create(&fixture.references());
    fs::write(
        fixture.path("01.genes.tsv"),
        "gene\tchr\tstart\tstop\nG_B\tchr2\t30\t40\nG_A\tchr1\t10\t20\n",
    )
    .unwrap();
    fs::write(
        fixture.path("01.tsv"),
        "gene\tc_a\tc_obs\tc_b\nG_B\t2.5\t4\t6.5\nG_A\t-0\t3.75\t5.125\n",
    )
    .unwrap();
    assert!(validate_created(&created, &fixture.expected().unwrap()).is_err());
}

#[test]
fn changed_cell_column_order_is_rejected() {
    let fixture = Fixture::new(false, false);
    let created = fixture.create(&fixture.references());
    fs::write(
        fixture.path("01.cells.tsv"),
        "cell\tgroup\trole\nc_obs\to\tobservation\nc_a\tr_a\treference\nc_b\tr_b\treference\n",
    )
    .unwrap();
    fs::write(
        fixture.path("01.tsv"),
        "gene\tc_obs\tc_a\tc_b\nG_A\t3.75\t-0\t5.125\nG_B\t4\t2.5\t6.5\n",
    )
    .unwrap();
    fs::write(
        fixture.path("expected.maps.tsv"),
        "role\tgroup\tindex\tcell\nreference\tr_b\t3\tc_b\nreference\tr_a\t2\tc_a\nobservation\to\t1\tc_obs\n",
    ).unwrap();
    assert!(validate_created(&created, &fixture.expected().unwrap()).is_err());
}

#[test]
fn existing_empty_export_directory_is_rejected() {
    let fixture = Fixture::new(false, false);
    let created = fixture.create(&fixture.references());
    let output = fixture.path("existing");
    fs::create_dir(&output).unwrap();
    assert!(write_creation(&output, &created).is_err());
    assert_eq!(fs::read_dir(output).unwrap().count(), 0);
}

#[test]
fn existing_nonempty_export_directory_is_preserved() {
    let fixture = Fixture::new(false, false);
    let created = fixture.create(&fixture.references());
    let output = fixture.path("existing");
    fs::create_dir(&output).unwrap();
    fs::write(output.join("sentinel"), "keep").unwrap();
    assert!(write_creation(&output, &created).is_err());
    assert_eq!(fs::read_to_string(output.join("sentinel")).unwrap(), "keep");
    assert_eq!(fs::read_dir(output).unwrap().count(), 1);
}

#[test]
fn repeated_export_preserves_all_previous_bytes() {
    let fixture = Fixture::new(false, false);
    let created = fixture.create(&fixture.references());
    let output = fixture.path("export");
    write_creation(&output, &created).unwrap();
    let files = ["expression.tsv", "genes.tsv", "cells.tsv", "maps.tsv"];
    let before: Vec<_> = files
        .iter()
        .map(|file| fs::read(output.join(file)).unwrap())
        .collect();
    assert!(write_creation(&output, &created).is_err());
    let after: Vec<_> = files
        .iter()
        .map(|file| fs::read(output.join(file)).unwrap())
        .collect();
    assert_eq!(before, after);
}

#[test]
fn missing_export_parent_returns_contextual_error() {
    let fixture = Fixture::new(false, false);
    let created = fixture.create(&fixture.references());
    let output = fixture.path("missing").join("export");
    let error = write_creation(&output, &created).unwrap_err();
    assert!(error.contains(&output.display().to_string()));
    assert!(!fixture.path("missing").exists());
}

macro_rules! separator_export {
    ($name:ident, $(($file:literal, $from:literal, $to:literal)),+ $(,)?) => {
        #[test]
        fn $name() {
            let fixture = Fixture::new(false, false);
            $(fixture.replace($file, $from, $to);)+
            let created = fixture.create(&fixture.references());
            let output = fixture.path("export");
            assert!(write_creation(&output, &created).is_err());
            assert!(!output.exists());
        }
    };
}

separator_export!(
    gene_separator_is_rejected_before_export_creation,
    ("counts.tsv", "G_A", "G\rA"),
    ("positions.tsv", "G_A", "G\rA")
);
separator_export!(
    chromosome_separator_is_rejected_before_export_creation,
    ("positions.tsv", "chr1", "ch\rr1")
);
separator_export!(
    cell_separator_is_rejected_before_export_creation,
    ("counts.tsv", "c_a", "c\ra"),
    ("annotations.tsv", "c_a", "c\ra")
);

#[test]
fn group_separator_is_rejected_before_export_creation() {
    let fixture = Fixture::new(false, false);
    fixture.replace("annotations.tsv", "r_a", "r\ra");
    let created = fixture.create(&["r_b".into(), "r\ra".into()]);
    let output = fixture.path("export");
    assert!(write_creation(&output, &created).is_err());
    assert!(!output.exists());
}

```

Append this exact Cargo.toml registration, before the existing bench declaration:

```toml
[[test]]
name = "cnv_ingestion_measurement_contract"
path = "tests/cnv_ingestion_measurement_contract.rs"
required-features = ["infercnv-measurement"]
```

- [x] **Step 2: Extend the existing candidate workflow without changing default/old selectors.**

Under dispatch inputs add:

```yaml
      ingestion_measurement:
        description: Test private exact raw-creation comparison and export support
        type: boolean
        default: false
```

Under native env add:

```yaml
      INGESTION_MEASUREMENT: ${{ inputs.ingestion_measurement }}
```

In `Verify and extract frozen source`, immediately after snapshot-name validation add:

```bash
if [[ "$INGESTION_MEASUREMENT" == true ]]; then
  test "$MEASUREMENT" = true
fi
```

Before `Test measurement support debug` add these dedicated steps:

```yaml
      - name: Test ingestion support debug
        if: ${{ !cancelled() && inputs.ingestion_measurement && steps.metadata.outcome == 'success' }}
        working-directory: ${{ runner.temp }}/sc-cnv-candidate
        run: |
          set -euo pipefail
          cargo test --locked --features infercnv-measurement --test cnv_ingestion_measurement_contract \
            -- --nocapture 2>&1 | tee "$EVIDENCE/ingestion-debug.log"

      - name: Test ingestion support release
        if: ${{ !cancelled() && inputs.ingestion_measurement && !inputs.expected_red && steps.metadata.outcome == 'success' }}
        working-directory: ${{ runner.temp }}/sc-cnv-candidate
        run: |
          set -euo pipefail
          cargo test --locked --release --features infercnv-measurement --test cnv_ingestion_measurement_contract \
            -- --nocapture 2>&1 | tee "$EVIDENCE/ingestion-release.log"
```

Replace the host.log selector record with:

```bash
printf 'snapshot=%s\nexpected_red=%s\nmeasurement=%s\ningestion_measurement=%s\nresolve_measurement_lock=%s\n' \
  "$SNAPSHOT_ID" "$EXPECT_RED" "$MEASUREMENT" "$INGESTION_MEASUREMENT" "$RESOLVE_MEASUREMENT_LOCK"
```

Validate YAML with Ruby/Psych and parse every run block with Bash `-n`; test the extracted actual guard with all four true/false combinations. Expected: only ingestion=true/measurement=false fails, and old default dispatch behavior is unchanged.

- [x] **Step 3: Freeze RED without running local Cargo.**

Use external Rust 1.91 standalone rustfmt with `--edition 2024 --config skip_children=true` on the new test. Verify baseline source/old support/lock hashes against green-10's manifest excluding only Cargo.toml. Generate `files.sha256` from sorted owning-repo `git ls-files --cached --others --exclude-standard`, `source.tar.gz` from exactly those files, and `checksums.sha256` for the source archive and manifest. Add README through apply_patch: baseline e62fc960, new opt-in test/registration only, helper absent, intended E0583, no timing/acceptance. Refuse an already-existing snapshot directory instead of overwriting it.

The exact generation commands, from the owning repository after creating the
new snapshot README via apply_patch, are:

```bash
TMPDIR=/Volumes/KIOXIA/Developments/tmp /Volumes/KIOXIA/Developments/rustup-home/toolchains/1.91.0-aarch64-apple-darwin/bin/rustfmt --edition 2024 --config skip_children=true tests/cnv_ingestion_measurement_contract.rs
test ! -e benches/ingestion_support/mod.rs
awk '$2 != "Cargo.toml"' "/Volumes/Zane's HDD/Documents/rsomics-world/.autopilot/snapshots/sc-cnv-core-2026-09-26/green-10/files.sha256" | shasum -a 256 --check
test ! -e "/Volumes/Zane's HDD/Documents/rsomics-world/.autopilot/snapshots/sc-cnv-core-2026-09-26/red-7/source.tar.gz"
git ls-files --cached --others --exclude-standard | LC_ALL=C sort | while IFS= read -r task_source; do shasum -a 256 "$task_source"; done > "/Volumes/Zane's HDD/Documents/rsomics-world/.autopilot/snapshots/sc-cnv-core-2026-09-26/red-7/files.sha256"
awk '{print substr($0,67)}' "/Volumes/Zane's HDD/Documents/rsomics-world/.autopilot/snapshots/sc-cnv-core-2026-09-26/red-7/files.sha256" > /Volumes/KIOXIA/Developments/tmp/rsomics-ingestion-support-proposal-20260930/red-7-files.list
tar -czf "/Volumes/Zane's HDD/Documents/rsomics-world/.autopilot/snapshots/sc-cnv-core-2026-09-26/red-7/source.tar.gz" -T /Volumes/KIOXIA/Developments/tmp/rsomics-ingestion-support-proposal-20260930/red-7-files.list
```

From the new snapshot directory:

```bash
shasum -a 256 source.tar.gz files.sha256 > checksums.sha256
shasum -a 256 --check checksums.sha256
```

The generated files are evidence/archive outputs, not hand-edited source.
Snapshot README is the complete tracked `red-7/README.md`. Source lists must
first be checked for only the two owned modifications and absent helper.

- [x] **Step 4: Verify controller tests, commit exact world plan/workflow/RED state, push and wait for exact-head Control plane CI.**

```bash
TMPDIR=/Volumes/KIOXIA/Developments/tmp PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest discover -s scripts -p 'test_*.py'
python3 -B scripts/validate_control_plane.py
git diff --check
```

Expected: all 223 existing controller tests pass, architecture consistent, no whitespace errors. Root stages only owned world files/snapshot; inherited VCF/scratch stays untouched. Commit subject `test(sc): freeze exact creation support contracts`. Existing product remains uncommitted until GREEN/review.

After exact-head CI passes dispatch:

```bash
gh workflow run sc-cnv-core-candidate.yml --ref main -f snapshot=red-7 -f expected_red=true -f measurement=true -f ingestion_measurement=true -f oracle_receipt=none
```

Expected: Linux x86_64 metadata/source/dependencies succeed; dedicated new test
compilation fails specifically because `tests/../benches/ingestion_support/mod.rs`
is absent. The actual Rust 1.91 path-attribute diagnostic is `couldn't read`
with `No such file or directory (os error 2)`, not E0583. Original
API/log/archive/CRC/path/source checks and ordinary tests must distinguish
this intended RED from environment or syntax failure. Preserve the complete
failure before implementation. The original RED README's E0583 expectation
remains historical; the exact diagnostic correction is ledgered, not hidden.

### Task 2: Implement the private helper against the observed RED

**Files:** create owning `benches/ingestion_support/mod.rs` only.

**Interfaces:** consumes checked CreatedInput getters and trusted `ExpectedStage::load(full_dir, 1)`; produces exactly the test contract's private type/getter/functions. Roles derive from actual reference indices, not shipped reference-name constants.

- [x] **Step 1: Write this complete minimal implementation after the genuine RED.**

```rust
#[allow(
    dead_code,
    reason = "only the single-stage loader is needed from the shared ten-stage fixture API"
)]
#[path = "../../tests/support/mod.rs"]
mod oracle;

use oracle::ExpectedStage;
use rsomics_sc::cnv::{CellGroup, CreatedInput, PreparedCounts};
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::Path;

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Reference,
    Observation,
}

impl Role {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "reference" => Ok(Self::Reference),
            "observation" => Ok(Self::Observation),
            _ => Err(format!("invalid role {value:?}")),
        }
    }
}

struct ExpectedGroup {
    name: String,
    indices: Vec<usize>,
}

pub struct ExpectedCreation {
    stage: ExpectedStage,
    references: Vec<ExpectedGroup>,
    observations: Vec<ExpectedGroup>,
}

impl ExpectedCreation {
    pub fn reference_names(&self) -> Vec<String> {
        self.references
            .iter()
            .map(|group| group.name.clone())
            .collect()
    }
}

fn literal(value: &str) -> Result<()> {
    if value
        .bytes()
        .any(|byte| matches!(byte, b'\t' | b'\n' | b'\r'))
    {
        return Err("literal TSV field contains a separator".into());
    }
    Ok(())
}

fn rows(path: &Path, header: &str, width: usize) -> Result<Vec<Vec<String>>> {
    let text = fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut lines = text.lines();
    if lines.next() != Some(header) {
        return Err(format!("{}: invalid header", path.display()));
    }
    lines
        .enumerate()
        .map(|(index, line)| {
            let fields: Vec<String> = line.split('\t').map(str::to_owned).collect();
            if fields.len() != width
                || fields
                    .iter()
                    .any(|field| field.is_empty() || literal(field).is_err())
            {
                return Err(format!("{}: invalid row {}", path.display(), index + 2));
            }
            Ok(fields)
        })
        .collect()
}

fn validate_stage(root: &Path, stage: &ExpectedStage) -> Result<()> {
    PreparedCounts::new(
        stage.genes().to_vec(),
        stage.cells().to_vec(),
        stage.values().to_vec(),
    )
    .map_err(|error| format!("{}: invalid expected creation: {error}", root.display()))?;

    let path = root.join("01.genes.tsv");
    let gene_rows = rows(&path, "gene\tchr\tstart\tstop", 4)?;
    for (index, row) in gene_rows.iter().enumerate() {
        for token in [&row[2], &row[3]] {
            if !token.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(format!(
                    "{}: non-decimal coordinate at row {}",
                    path.display(),
                    index + 2
                ));
            }
        }
    }
    for gene in stage.genes() {
        literal(&gene.id)?;
        literal(&gene.chromosome)?;
    }
    for cell in stage.cells() {
        literal(&cell.id)?;
        literal(&cell.group)?;
    }
    Ok(())
}

pub fn load_expected(full_dir: &Path, maps_path: &Path) -> Result<ExpectedCreation> {
    let stage = ExpectedStage::load(full_dir, 1)?;
    validate_stage(full_dir, &stage)?;
    let cells = stage.cells();

    let path = full_dir.join("01.cells.tsv");
    let cell_rows = rows(&path, "cell\tgroup\trole", 3)?;
    if cell_rows.len() != cells.len() {
        return Err(format!("{}: cell count mismatch", path.display()));
    }
    let mut roles = Vec::with_capacity(cells.len());
    for (index, (row, cell)) in cell_rows.iter().zip(cells).enumerate() {
        if row[0] != cell.id || row[1] != cell.group {
            return Err(format!(
                "{}: identity mismatch at row {}",
                path.display(),
                index + 2
            ));
        }
        roles.push(
            Role::parse(&row[2])
                .map_err(|error| format!("{}: row {}: {error}", path.display(), index + 2))?,
        );
    }

    let map_rows = rows(maps_path, "role\tgroup\tindex\tcell", 4)?;
    let mut references: Vec<ExpectedGroup> = Vec::new();
    let mut observations: Vec<ExpectedGroup> = Vec::new();
    let mut names = HashSet::new();
    let mut covered = vec![false; cells.len()];
    let mut observations_started = false;

    for (row_index, row) in map_rows.iter().enumerate() {
        let fail =
            |message: &str| format!("{}: row {}: {message}", maps_path.display(), row_index + 2);
        let role = Role::parse(&row[0]).map_err(|error| fail(&error))?;
        if !row[2].bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(fail("invalid one-based index"));
        }
        let index = row[2]
            .parse::<usize>()
            .ok()
            .and_then(|value| value.checked_sub(1))
            .filter(|&value| value < cells.len())
            .ok_or_else(|| fail("index out of range"))?;
        if cells[index].id != row[3] || cells[index].group != row[1] || roles[index] != role {
            return Err(fail("cell, group or role mismatch"));
        }
        if covered[index] {
            return Err(fail("duplicate cell coverage"));
        }
        covered[index] = true;

        let groups = match role {
            Role::Reference => {
                if observations_started {
                    return Err(fail("reference follows observation"));
                }
                &mut references
            }
            Role::Observation => {
                observations_started = true;
                &mut observations
            }
        };
        let starts_group = groups.last().is_none_or(|group| group.name != row[1]);
        if starts_group {
            if !names.insert(row[1].clone()) {
                return Err(fail("split or cross-role group"));
            }
            if role == Role::Observation
                && groups
                    .last()
                    .is_some_and(|group| group.name.as_bytes() >= row[1].as_bytes())
            {
                return Err(fail("unordered observation groups"));
            }
            groups.push(ExpectedGroup {
                name: row[1].clone(),
                indices: Vec::new(),
            });
        }
        let group = groups.last_mut().unwrap();
        if group
            .indices
            .last()
            .is_some_and(|&previous| previous >= index)
        {
            return Err(fail("unordered group members"));
        }
        group.indices.push(index);
    }

    if covered.iter().any(|&covered| !covered) {
        return Err(format!("{}: incomplete cell coverage", maps_path.display()));
    }
    Ok(ExpectedCreation {
        stage,
        references,
        observations,
    })
}

fn compare_groups(role: &str, actual: &[CellGroup], expected: &[ExpectedGroup]) -> Result<()> {
    if actual.len() != expected.len() {
        return Err(format!("{role} group count differs"));
    }
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        if actual.name() != expected.name || actual.cell_indices() != expected.indices {
            return Err(format!("{role} group differs at position {}", index + 1));
        }
    }
    Ok(())
}

pub fn validate_created(created: &CreatedInput, expected: &ExpectedCreation) -> Result<()> {
    let actual = created.prepared_counts().state();
    if actual.genes() != expected.stage.genes() {
        return Err("creation genes or coordinates differ".into());
    }
    if actual.cells() != expected.stage.cells() {
        return Err("creation cells or groups differ".into());
    }
    if actual.values().len() != expected.stage.values().len() {
        return Err("creation expression length differs".into());
    }
    for (index, (&actual, &expected)) in actual
        .values()
        .iter()
        .zip(expected.stage.values())
        .enumerate()
    {
        if actual.to_bits() != expected.to_bits() {
            return Err(format!("creation count bits differ at offset {index}"));
        }
    }
    compare_groups(
        "reference",
        created.reference_groups(),
        &expected.references,
    )?;
    compare_groups(
        "observation",
        created.observation_groups(),
        &expected.observations,
    )
}

fn emit(path: &Path, write: impl FnOnce(&mut BufWriter<File>) -> io::Result<()>) -> Result<()> {
    let result = (|| {
        let file = OpenOptions::new().write(true).create_new(true).open(path)?;
        let mut writer = BufWriter::new(file);
        write(&mut writer)?;
        writer.flush()
    })();
    result.map_err(|error| format!("{}: {error}", path.display()))
}

pub fn write_creation(new_output_dir: &Path, created: &CreatedInput) -> Result<()> {
    let state = created.prepared_counts().state();
    for gene in state.genes() {
        literal(&gene.id)?;
        literal(&gene.chromosome)?;
    }
    for cell in state.cells() {
        literal(&cell.id)?;
        literal(&cell.group)?;
    }

    let mut roles = vec!["observation"; state.cell_count()];
    for group in created.reference_groups() {
        for &index in group.cell_indices() {
            roles[index] = "reference";
        }
    }
    fs::create_dir(new_output_dir)
        .map_err(|error| format!("{}: {error}", new_output_dir.display()))?;

    emit(&new_output_dir.join("genes.tsv"), |writer| {
        writeln!(writer, "gene\tchr\tstart\tstop")?;
        for gene in state.genes() {
            writeln!(
                writer,
                "{}\t{}\t{}\t{}",
                gene.id, gene.chromosome, gene.start, gene.end
            )?;
        }
        Ok(())
    })?;
    emit(&new_output_dir.join("cells.tsv"), |writer| {
        writeln!(writer, "cell\tgroup\trole")?;
        for (cell, role) in state.cells().iter().zip(&roles) {
            writeln!(writer, "{}\t{}\t{role}", cell.id, cell.group)?;
        }
        Ok(())
    })?;
    emit(&new_output_dir.join("expression.tsv"), |writer| {
        write!(writer, "gene")?;
        for cell in state.cells() {
            write!(writer, "\t{}", cell.id)?;
        }
        writeln!(writer)?;
        for (gene_index, gene) in state.genes().iter().enumerate() {
            write!(writer, "{}", gene.id)?;
            for cell_index in 0..state.cell_count() {
                let value = state.values()[cell_index * state.gene_count() + gene_index];
                write!(writer, "\t{value:.17e}")?;
            }
            writeln!(writer)?;
        }
        Ok(())
    })?;
    emit(&new_output_dir.join("maps.tsv"), |writer| {
        writeln!(writer, "role\tgroup\tindex\tcell")?;
        for (role, groups) in [
            ("reference", created.reference_groups()),
            ("observation", created.observation_groups()),
        ] {
            for group in groups {
                for &index in group.cell_indices() {
                    writeln!(
                        writer,
                        "{role}\t{}\t{}\t{}",
                        group.name(),
                        index + 1,
                        state.cells()[index].id
                    )?;
                }
            }
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn writer_error_keeps_partial_file_and_path_context() {
        let root = std::env::temp_dir().join(format!(
            "rsomics-ingestion-writer-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let path = root.join("partial.tsv");
        let error = emit(&path, |writer| {
            writeln!(writer, "partial")?;
            writer.flush()?;
            Err(io::Error::other("injected writer failure"))
        })
        .unwrap_err();
        assert!(error.contains(&path.display().to_string()));
        assert!(error.contains("injected writer failure"));
        assert_eq!(fs::read_to_string(&path).unwrap(), "partial\n");
        fs::remove_dir_all(&root).unwrap();
    }
}

```

- [x] **Step 2: Format the two new files using external Rust 1.91 rustfmt, and freeze green-11.**

Keep Cargo.lock, old support/tests/bench and all `src/` bytes identical to baseline. Build the new immutable archive/manifest/checksums as in Task 1, with README stating private support only and no timed factory/performance claim. Test logs/source after the run must match this same archive, not a subsequent edited version.

Root artifact audit of green-11 discovered a packaging defect: default macOS
tar added 146 undeclared AppleDouble entries. Its code tests passed, but the
archive cannot satisfy the existing strict source verifier. Preserve red-7
and green-11 unchanged; freeze green-12 with identical source manifest using:

```bash
COPYFILE_DISABLE=1 tar --no-xattrs --format=ustar -czf "/Volumes/Zane's HDD/Documents/rsomics-world/.autopilot/snapshots/sc-cnv-core-2026-09-26/green-12/source.tar.gz" -T /Volumes/KIOXIA/Developments/tmp/rsomics-ingestion-support-proposal-20260930/green-11-files.list
```

From green-12, hash README.md, files.sha256 and source.tar.gz into the checksum
file. Verify exactly 146 unique safe regular archive members, every manifest
hash and no metadata extras before extraction. The separate source-freeze
guard plan adds this same pre-extraction check to the candidate workflow;
then use `snapshot=green-12` in the Task 3 dispatch. No source/test or numerical
gate is changed; original uploads and failed packaging assertions remain.

- [x] **Step 3: Root reviews complete source/test and requests a fresh independent code review of this owning-repo diff against e62fc960.**

Review expected-role boundary, coverage/order, bit comparisons, 17e roundtrip, field preflight and every opening/write/flush error path. Resolve critical/important findings with failing regression then minimal fix; preserve all deviations in the measurement ledger. Review approves code only until native evidence arrives.

### Task 3: Four-native gate and scoped acceptance

**Files:** freeze/commit world green-11 and ledger; subsequently commit only the three owned product paths after native acceptance.

**Interfaces:** consumes exact helper/tests and candidate selector; produces a hash-pinned private-support acceptance, not an actual measured executable or a product release.

- [ ] **Step 1: Re-run controller checks, push only owned world candidate files and verify exact-head CI before dispatch.**

Expected: unchanged 223 controller tests and architecture check pass. Then run:

```bash
gh workflow run sc-cnv-core-candidate.yml --ref main -f snapshot=green-11 -f expected_red=false -f measurement=true -f ingestion_measurement=true -f oracle_receipt=infercnv-shipped-2026-09-26
```

Expected: four native target jobs each pass new helper tests and old measurement support in debug/release, ordinary and explicit external-oracle tests, old bench compile, source-before/after and Linux format/strict Clippy gates. Empty or skipped dedicated logs are not acceptance. Retain original run/jobs/artifact API responses, zipped logs/artifacts and hashes, verify safe inventories/CRCs, frozen archive/manifest/lock and unchanged production source; root audits actual named test results rather than trusting a green UI.

- [ ] **Step 2: Commit only owning Cargo.toml, new test and helper on main.**

```bash
git add Cargo.toml tests/cnv_ingestion_measurement_contract.rs benches/ingestion_support/mod.rs
git diff --cached --check
git commit -m 'test(sc): support exact creation factory exports'
```

Owning product currently has no remote; do not create/publish an empty delivery repository to satisfy a mechanical push step. World evidence is pushed and exact-head tested. Record exact product/candidate/source hashes and limits in the tracked world measurement ledger; input-only receipt remains distinct from native support and later timing acceptance.

- [ ] **Step 3: Continue the actual factory trial/paired driver under their own complete plans.**

This helper does not authenticate whole-trial counters/provenance or time any operation. Next work must preserve all 32 fresh processes/64 warm+measured outputs, actual package/source pins and predetermined independent plain/gzip strict-wall gates. Native samples/Ward, default Leiden/HMM and unified-help full delivery remain open.
