# Genomic feature-count product gate — 2026-08-02

Status: `rsomics-count 0.1.0` published and independently verified.

## Released contract

The release provides one coherent count-table product:

- the default command assigns SAM or BAM reads and fragments to GTF, GFF3, or
  SAF features and meta-features;
- one invocation accepts multiple libraries and writes one count matrix plus a
  complete assignment summary;
- assignment supports single and paired units, three strand modes, checked
  overlap thresholds, feature and meta-feature levels, multi-overlap and
  multi-mapping exclusion/full/fractional policies, chromosome aliases, MAPQ,
  primary, duplicate, split, mate, chimera, and fragment-length filters;
- `normalize` reads a strict gene-by-sample matrix and computes TPM, FPKM, or
  FPKM-UQ with explicit positive lengths and nonzero denominators.

Named outputs use the paired and single-file transactions in
`rsomics-common`. Parse, annotation, CIGAR, tag, pairing, I/O, normalization,
and compatibility failures reach the top-level command and return nonzero.
CRAM, read-position projection, junction counts, read-group stratification,
long-read mode, per-record assignments, largest-overlap, non-overlap limits,
and extra annotation output attributes are absent rather than placeholders.

## Exact identities

- release source and VCS identity:
  `5bc764d8af75171c4ac2e4a9e4a153d0e2d05b49`;
- benchmarked implementation source:
  `3a0154b15c8a985b999a98c4dd61d07a7533dec3`;
- exact-head four-native-target CI: `30739798218`;
- publish workflow: `30739909672`;
- crates.io archive SHA-256:
  `5ab56387a05dca92a083beef625acfcdea51c24684e2e6de7c69616d3be42559`;
- crates.io archive size: 49,106 bytes;
- installed registry binary SHA-256:
  `c3fe679f17e465c791029fbb791e5b4ad27be22ef1417645184126e6a7044e5b`.

The crates.io API reports 0.1.0 as non-yanked. An independently downloaded
archive matched the registry checksum and its `.cargo_vcs_info.json` named the
exact release head.

## Compatibility and failure evidence

The always-run suite has 13 library tests, one CLI-tree test, 12 format and
failure-atomicity tests, three normalization tests, and six frozen
featureCounts oracle groups. It covers strict GTF, GFF3, SAF, gzip and alias
paths; raw SAM and BAM; stdin; malformed and truncated inputs; coordinate- and
query-name-sorted pairing; missing names and invalid tags; multiple libraries;
deterministic parallel fractional output; and preservation of existing paired
outputs on failure.

The live differential exercises featureCounts 2.1.1 defaults plus overlap,
fractional overlap, multi-mapping, combined fractions, minimum and fractional
thresholds, split modes, strand modes, filters, feature level, paired and
coordinate-sorted fragments, mate and fragment-length checks, paired strand,
paired multi-mapping, CIGAR edges, and primary-only behavior. It compares every
normalized count row and every assignment-summary category.

CI downloads official archives, verifies pinned SHA-256 values, and runs the
live differential on native Linux x86_64, macOS x86_64, and macOS aarch64.
Linux aarch64 runs the complete frozen suite because Subread does not publish
an aarch64 Linux archive. All four targets run debug and release tests on Rust
1.91; Linux x86_64 additionally passes formatting, strict Clippy, rustdoc with
warnings denied, and package reconstruction.

## Foundation decisions

The product uses registry releases `rsomics-common 0.12.0`,
`rsomics-help 0.4.0`, `rsomics-bamio 0.5.0`, and `rsomics-intervals 0.3.0`.
It is a concrete consumer of `bamio`'s borrowed raw BAM record path, common's
runtime and paired-output transaction, help's shared CLI presentation, and
intervals' checked geometry.

Counting policy, annotation grouping, assignment categories, normalization,
and feature indexes remain private product modules. No annotation, assignment,
count-matrix, or normalization foundation was added because a second product
consumer has not demonstrated a policy-free shared contract.

The historical CIGAR projection, lookup, and default assignment assets were
refactored into the product's typed paths. Historical parsers, output code, and
normalization behavior served as tests or negative design evidence where they
silently skipped errors, summed overlapping lengths, overwrote multiple
libraries, or defaulted missing lengths. Duplicated runtime and help code was
discarded. No retired micro-crate repository was revived.

## Representative performance

The tracked evidence is
[`benchmarks/2026-08-02`](https://github.com/omics-rust/rsomics-count/tree/main/benchmarks/2026-08-02).
It records source, binary, oracle and fixture hashes, exact command families,
all hyperfine samples, CPU times, `/usr/bin/time -l` RSS samples, and normalized
output hashes.

The Apple M2 fixtures contain two million single-end BAM records or one million
paired fragments over 500 genes and 1,500 exons. Normalized matrices and every
summary column matched featureCounts 2.1.1 in the single, paired, fractional
policy, and two-library cases.

The implementation does not have a throughput advantage. End-to-end wall time
was 1.67 times featureCounts at one thread and 2.52 times at four threads for
single reads; four-thread paired, fractional-policy, and two-library ratios
were 2.56, 2.37, and 1.75. It used less total CPU in the four-thread single,
policy, and two-library cases, but more in the one-thread and paired cases.

Median peak RSS for the four-thread single case was 68,108,288 bytes for
featureCounts and 8,503,296 bytes for rsomics, an 87.5% reduction. Paired
medians were 68,222,976 and 8,634,368 bytes, an 87.3% reduction. This material
resource advantage, not throughput, satisfies the first-release performance
gate. Throughput remains an explicit optimization target.

## Publication verification

An isolated external-disk `cargo install --locked rsomics-count@0.1.0`
succeeded. The installed binary reported 0.1.0 and exposed the expected shared
help surface. Its count and TPM smoke produced assignment total 5 and TPM
values 571428.571429, 238095.238095, and 190476.190476; the output SHA-256
values were respectively
`b5d981e97d86789fc54215755fbf4f71645e9701e536fd90a5dd89d138f3c46d`,
`fd463ba35b9a1a945bada272ad99a6e2c9775174acf359173aa983e481f56828`,
and `ae5e6f65cbcf5ecc03024bc5b99a70e34f93a50f2885dbc9a005166f0720e2c7`.

The registry token was exposed to the repository only for publish workflow
`30739909672`; repository access was removed immediately after success and
verified absent.
