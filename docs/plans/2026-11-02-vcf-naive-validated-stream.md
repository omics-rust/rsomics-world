# VCF naive validated-stream repair

> **For agentic workers:** Use `superpowers:executing-plans` for this coherent repair, with one fresh whole-feature review. No helper-by-helper review or approval ladder.

**Goal:** Make the existing private `concat --naive` path accept legal BGZF extended headers while retaining complete typed validation before output.

**Architecture:** Reuse the product's existing `format::bgzf::DecodedReader` and `inflate_frame`. One validated typed preflight per input is followed by the existing raw-copy pass. Format policy stays inside VCF; no dependency, public API, foundation, or command boundary changes.

**Tech stack:** Rust 1.91, pinned Noodles/flate2, existing VCF/BCF record and header types.

**Spec:** `docs/plans/2026-08-19-vcf-concat-design.md`, especially naive mode and repair execution step 3; inherited `.autopilot/state/vcf-naive-preflight-2026-09-09.md` is a protected historical proposal, not a file to update.

## Global constraints

- Owner is `/Volumes/KIOXIA/Documents/omics-rust/rsomics-vcf`, currently `682942cfa69768dc3a127a8544f2f07213b704ea` with substantial inherited uncommitted work. Preserve its empty index, inherited bodies, and frozen snapshots. Do not commit the inherited product wholesale during this task.
- Before builds, use a fully inspected external-path guard, physically verify KIOXIA and resolved Cargo/Rustup/target/temp/tool-cache paths, and retain at least 8 GiB free. Cargo target is `/Volumes/KIOXIA/Developments/cargo-target/rsomics-vcf`; scratch is below `/Volumes/KIOXIA/Developments/tmp`. No boot-disk output.
- Ordinary `Reader::open`, indexed/reheader behavior, CLI, Cargo manifest/lock, public help and version remain unchanged. Rare invariant comments only.
- Retain checked BCF record boundaries, typed conversion, complete header and BCF dictionary equality, CRC/ISIZE/size checks, one terminal EOF, and no trailing bytes. Do not replace typed completion with a blind drain.
- No release, registry action, destructive cleanup, new public BGZF API, source-oracle waiver, or speed claim. Reducing passes is not performance evidence.
- Full concat differential/performance and BCF-reheader resource gates remain separate unfinished work. Snapshot native CI does not establish owning exact-head CI.

## Evidence and diagnosis

The 174 current source bodies match the retained naive-red snapshot. Actual run `34300381682` rejected all 32 legal extended-frame cases on every native platform while four canonical controls passed. Existing frame parsing accepts arbitrary extra subfields, but naive's local inflater re-enters Noodles' fixed 18-byte header reader. Ordinary format sniffing also uses only its current compressed buffer slice; a large legal extra or empty-frame prefix can exceed that slice. The first rejection masks this second limitation in the existing runtime matrix.

Read-only recovery report: `/Volumes/KIOXIA/Developments/tmp/vcf-concat-frontier-readonly-20261102.ltLyi4/REPORT.md`, SHA256 `0dca4eb1ebe6c7ad77aed4fbe57593dd5dcdab30d3ee46566f83d801ba937f82`.

## Review focus

- A typed loop must reach validated physical EOF, including a malformed second input or zero shared-length BCF record; failure must precede stdout bytes and preserve named output.
- Sniffing must work through short/Interrupted reads, split magic, long empty prefixes and large extras without retaining an unbounded compressed prefix.
- Header cutting must preserve first-input frames and later raw body frames, recompressing only the header-boundary suffix; exactly one output EOF remains.
- Text EOF without a final newline can differ between typed parsing and raw splicing. Characterize with the pinned oracle before selecting a repair policy; do not silently broaden this task's contract.
- Source changes are layered on an inherited dirty candidate. Preserve original snapshots and report exact new deltas instead of presenting the whole dirty tree as newly reviewed.

## Task 1: Complete the private validated two-pass path

**Files:**

- Modify `src/format/reader.rs`: private BGZF construction and narrow read-behavior tests.
- Modify `src/concat/naive.rs`: typed preflight, encoding selection and shared boundary inflation; migrate the existing split-magic test rather than discard its contract.
- Modify `tests/concat_cli.rs` only for missing consumer-visible regression coverage. Preserve its existing 32-positive/four-control group unchanged.
- External scratch: before/after manifests, exact diff, guard, command logs, ledger and review package. No unrelated owner files.

**Interfaces:**

- Consume existing `DecodedReader<R: Read>`, `inflate_frame(&[u8]) -> io::Result<Vec<u8>>`, `Reader::read_header`, `Reader::read_record`, and checked BCF EOF handling.
- Produce `pub(crate) fn Reader::open_bgzf(path: &Path) -> Result<Self>` plus an internal generic read constructor for real short/Interrupted streams. Use at most 16 decoded magic bytes, replay them into the same stream, and reuse `Inner::Vcf` / `Inner::BcfRaw`; accept exact BCF 2.2 or the current VCF prefix.
- Naive derives its private Encoding from the typed reader. Its existing `write` signature and summary contract do not change.

- [ ] Capture the current 174-body/mode identity and empty Git index. Read the actual format reader, private BGZF decoder, naive copier and existing CLI regressions before editing.
- [ ] Run the existing legal-extra CLI regression against unchanged production under the inspected external guard. Expected: the 32 legal cases fail and four controls pass, not an environment/setup failure. Preserve the result.
- [ ] Add only missing focused behavior tests: decoded sniff through fragmented/Interrupted input and long empty/large-extra prefixes; late corrupt payload and invalid typed second input before output. Reuse existing CRC/EOF/header/dictionary/zero-length coverage rather than duplicate it. Expected red must name the missing behavior; no per-helper test campaign.
- [ ] Implement the narrow constructor and naive two-pass integration. Remove `inspect_frames`, its prefix detector and local fixed-layout inflater after their contracts are moved. Keep ordinary open and existing shared decoder unchanged unless a demonstrated defect requires a separately ledgered decision.
- [ ] Run the unchanged legal-extra group and the complete concat CLI suite. Expected: all 32 legal cases and four controls pass, existing malformed/transaction/BCF boundary cases stay green, and raw compressed prefix/tail assertions remain intact.
- [ ] Run formatting, strict Clippy and full debug/release suites once per final coherent candidate, recording actual ignored oracle counts separately. Run existing concat differentials against the pinned installed bcftools 1.24; absence is a reported gate, not a successful skip.
- [ ] Perform one fresh whole-feature review of the exact new delta plus inherited surrounding code relevant to this path. Fix material findings in one pass; record minor/deferred findings and every declined scope. No second review ladder.
- [ ] Record exact hashes, commands, failures, oracle version, coverage limits and remaining gates in a new state record. Product integration/publication and representative performance remain pending; do not claim the whole concat product complete.

## Rulings and next work

This is an authorized bounded repair under the user's unattended portfolio goal, not an independent foundation phase. No new approval checkpoint is needed for normal implementation. If newline characterization reveals a material compatibility choice, hold that affected behavior, record the evidence, and continue the unambiguous extended-header repair.

After this repair, complete the declared concat oracle matrix and representative ordered/indexed/naive/ligation performance and BCF-reheader resource coverage, then integrate the complete owning product with four native exact-head CI and delivery checks. Do not publish the frozen 0.6.0 snapshot with the known index byte-path defect.
