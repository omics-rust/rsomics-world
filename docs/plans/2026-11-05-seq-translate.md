# Sequence translation implementation plan

> Execute inline on the authorized owning main, with one whole-feature
> review and a targeted fix pass if needed.

**Goal:** Add a complete nucleotide-to-protein workflow to `rsomics-seq`,
covering FASTA/FASTQ input, selected or all signed frames, and the 24 genetic
codes used by the pinned SeqKit 2.13.0 profile.

**Architecture:** Product-private codon decoding under `src/operations/translate`,
with typed code/frame selection, one streaming scan, bounded reusable buffers,
and existing shared CLI, compressed output and transaction boundaries.

**Tech stack:** Rust1.91, existing rsomics-common/help/seqio, no new dependency.

**Spec:** `.autopilot/state/seq-translate-frontier-2026-11-05.md`,
`docs/10-products/sequence-fastq.md`, actual SeqKit v2.13.0 and its
bio v0.13.9 codon behavior.

## Scope and fixed decisions

- Owner starts at `b3d045affba732f2075cac4fc164bf08e204d220`. No new public
  foundation, micro-crate, upstream code import or unpublished version bump.
- Preserve whole header and protein FASTA bytes, frame ordering and default
  width60. Frame6 expands to 1,2,3,-1,-2,-3. Support explicit width0, frame
  suffix, DNA initiator conversion, stop cleaning and unknown codon policy.
  Use native --min-protein-length for full-translation filtering; source
  --min-len only applies to the excluded stop-delimited workflow. Error on
  malformed input and preserve named output on failure.
- Source 24-code profile is exact, not a claim of current NCBI equivalence.
  Derive concrete rows from factual NCBI4.6 data and the verified 3,072-case
  differential. Table3 does not recode initial GTG in this pinned profile.
  Do not silently adopt NCBI-only codes15,32,33.
- No error-skipping or stop-delimited ORF workflow is advertised.
  `--trim` remains held until the user resolves the documented-versus-actual
  truncation; it does not constrain the unaffected core.
- With uracil present, positive frames without initiator conversion are in
  scope. The pinned source leaves U and asymmetric IUPAC symbols unchanged
  on the reverse complement and misses valid RNA starts. Hold integration
  of those disputed results until the compatibility policy is chosen.
- First build one efficient streaming implementation. Measure representative
  throughput and process RSS before proposing a new worker or CLI layer.
- Guard real external devices, scratch and resolved Cargo target before
  builds, tests, or measurements; retain old work and raw evidence.

## Task 1: connected translation

- [x] Add complete output differentials to `tests/seqkit_compat.rs` with
  pinned FASTA/FASTQ, RNA, ambiguity/gaps, signed frames/wrapping, alternate
  code, initiators and a single all-table data witness. Run before production
  changes: failure must be an unrecognized translate subcommand.
- [x] Implement `translate_sequences(input, options, output)` and typed
  `GeneticCode`, `ReadingFrame`, `TranslateOptions`, `TranslateReport`.
  Read via the existing input scan. Resolve degenerate codons by agreement
  of concrete expansions, preserve full gaps, and encode negative frames
  without a whole-record reverse-complement allocation.
- [x] Wire `translate` in the shared CLI and existing output transactions;
  add one named compressed-output/rollback contract to the CLI tests.
  Prove complete byte matches and full suites with the actual oracle.
- [x] One independent whole-feature review for codon/start data, header and
  ordering, alias/late error and format boundaries. Fix only material issues
  in one pass. The unknown placeholder is aligned with the source; the
  documented reverse-IUPAC/RNA ruling remains open.

## Task 2: qualification and integration

- [x] Measure matched complete output on the SHA-verified 500,000-record,
  250-million-base workload with pinned SeqKit, resource evidence and an
  explicit throughput or memory decision. Local draft only; requalify any
  changed executable. No inferred parallel claim.
- [ ] Verify fmt, strict Clippy, docs, complete debug/release and clean package.
  Commit coherent source on main, push and verify exact four-native CI.
- [ ] Record verified scope and remaining hold in the dossier. Keep
  installation/publication, original artifact adoption and the held option
  distinct from source qualification.
