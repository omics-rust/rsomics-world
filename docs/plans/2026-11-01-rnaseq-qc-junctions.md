# RNA-seq QC junction annotation implementation plan

> For agentic workers: use subagent-driven-development or executing-plans for one coherent source-to-native wave, with one whole-change review per delivered wave, not per-helper review/test ladders. Root owns Git, CI dispatch and adoption. Read the task brief before implementation.

**Goal:** add junction detail, support and annotation summaries to the existing RNA-seq QC product, without changing its accepted default five metrics or creating another crate.

**Architecture:** a product-private accumulator consumes the existing original RawRecord and decoded CIGAR before pileup ownership transfer. It derives site indexes from the validated Annotation; bamio/common/help are reused and no foundation API is added.

**Tech stack:** Rust1.91; hosted Linux x86_64 CPython3.12.10; pinned RSeQC5.0.5 and the17 original version-pinned dependencies; stdlib control scripts and existing private-artifact transport.

**Spec:** RNA section of `docs/10-products/rnaseq-qc-signal.md`. The complete source/design report is `/Volumes/KIOXIA/Developments/tmp/rnaseq-junction-design-20261101.3W16Jt/REPORT.md`, SHA c01cea440806859131616d316e6a3f0d8dd2e7e6cb287890d12cb43b9c11261c; READ-IDENTITIES SHA00da83660d513071d76ca5a4c7396f316b1805cedc1995c03a9e0b302236c250. Root reads both completely and checks their identities before adoption.

## Global constraints

- Owner `/Volumes/KIOXIA/Documents/omics-rust/rsomics-rnaseq-qc` remains private/publish=false at clean91ba1d38156e09d878e04fcdf8600404551abdc0 before implementation. Donor `rsomics-bam-junctions` is4f7a39cd1a69c7bae5a5c98393ed15bcb0acd62e. Do not revive its public micro-crate.
- All Mac project writes/downloads/caches/scratch/builds physically use the external disks and existing guarded environment. No source/scientific package import, upstream execution or Linux ELF execution on Mac. No destructive cleanup.
- Default report keeps mapping/strandedness/distribution/coverage/integrity unchanged. Junctions is focused and explicitly opt-in; no empty saturation/plot/export command.
- Existing five-metric source/native/performance proofs are not junction proofs and are not rerun unchanged. New candidate integration must retain their existing tests/goldens.
- Reuse mandatory rsomics-help, common ThreadArgs/OutputArgs/run/AtomicFile and bamio original records. Pileup remains only for coverage/TIN, never a junction input. No new public foundation/dependency.
- Sparse comments, typed private interfaces, one BAM traversal, bounded workers and atomic output sets. Propagate parse/I/O/write errors; no silent annotation-row skipping.
- Root alone commits/stages/pushes/dispatches/publishes. Preserve inherited World VCF/untracked dirt and all earlier failed evidence. No automation, notifications, key-byte reads or plaintext-upload fallback.

## Review focus

- Junction legacy CIGAR differs from legacy distribution: S does not advance; =/X are ignored only in the explicit source profile.
- Independent pooled splice sites are not annotated intron-pair or strand membership; source chromosome pooling must not weaken existing global annotation validation.
- Count N events before the49/50/51bp minimum filter, distinguish repeated support from distinct junctions and retain supplementary/MAPQ policy.
- No-N/all-short source exits and partial files are captured as such; native complete-zero outputs must be documented as intentional differences, not false upstream success.
- Observe original records before pileup filtering/ownership transfer, keep old five outputs unchanged and include all new tables in the existing atomic transaction.

## Root decisions and execution rulings

The user's existing unattended/main/parallel authorization replaces repeated phase approvals and generic worktree/PR instructions. Use one implementer for the tightly coupled source witness and one final whole-change review; no worker-spawned reviewers, per-helper gates or automatic evidence deletion.

Fresh official standalone CPython3.12.10 is selected instead of the report's proposed3.12.15. Its existing official runtime authority is already available, but the new job must acquire/admit/retain it afresh. This is explicitly not restoration of the old3.12.15 runtime. All17 old dependency versions remain frozen; exact new wheels and installed bodies/providers must be pinned and qualified. No silent version substitution.

After actual source confirmation, valid no-N/all-short inputs should produce complete atomic native zero-result tables and accurate total/filtered counters with exit0. This is our implementation decision under delegated authority, recorded as an intentional source incompatibility; it is not another user-approved upstream repair or an observed native result yet. Other genuine source contradictions return to root, without changing expectations to fit them.

Initial native scope is annotation detail/summary plus report/manifest. Native R/PDF, BED/Interact exports, SAM/CRAM ingress and junction saturation are excluded, not advertised. Optional upstream exports receive one provenance capture only. Fair primary performance disables them in the source.

### Task 1: Prepare one real Linux junction source witness

**Files:** create `scripts/rnaseq_qc_junctions.py`, `.github/workflows/rnaseq-qc-junctions.yml`, `assets/rnaseq-qc-junctions-20261101.json`, `scripts/test_rnaseq_qc_junctions.py`. No owner or old oracle edits.

**Consumes:** existing source receipt `.autopilot/oracles/rnaseq-qc-first-workflow-2026-10-27.json` binds40520155303/1, artifact12740753197, ZIP897158B SHA b54f52612cd1bc8f0ea94f622ba24cba7379c9432c9914e78e185f2bc6ca1a97. Full source acquisition is retained under `/Volumes/Zane's HDD/rsomics-fixtures/evidence/rnaseq-qc-2026-10-27/run-40520155303/artifact/acquisition`.

**Produces:** manual-only contents/actions-read workflow, pinned policy and six actual junction CLI observations with complete runtime/input/output/status/argument identities and encrypted originals. Suggested witness schema is `rsomics.rnaseq-qc.junction-source.v1`; its actual run/head/artifact receipt is produced only after execution/adoption, never guessed.

- [ ] Write/run ONE connected stdlib preparation contract first; missing new workflow/controller is the expected initial RED. It covers the complete six-call plan, frozen source/runtime identities, meaningful fixture intent, expected nonzero retention and private finalization. No source imports or artificial native functional RED.
- [ ] Freeze the real RSeQC wheel204772B SHA a1fca6d455886b19bc2d58aa1e13b233977a2c835f676ef93bd0c6f4099b8452 and source TAR472500B SHA8c9d34c208bf7852f147dd365312a43a546148d1bf76581c5c0ede73fdb254b8, commit0d2e017723c5c4b41eb3b938e1631692d187de1f. Actual wheel script SHA527d4ef87cab543a5e92f5438f16152bc79f8a5f35f25a15b6ccefd0aa09e03b, SAM.py43727df48288f808159b1bf60c75db010a8e115359dd5280ddf3e0211944d5f5, bam_cigar.py805777b0853595733630db164663036f15f0e62e2af201f244385c652eafa2b9.
- [ ] Reuse the existing pinned GCC14.3 Bookworm manifest/config and official standalone3.12.10 identities. Obtain the17 exact dependency versions from installed-freeze.txt, excluding its separate RSeQC direct URL; retain official PyPI metadata, complete compatible CPython3.12/Linux wheel bodies and METADATA hashes before producing the frozen asset. Metadata/archive downloads may occur under the external guard, never installation/import on Mac. The hosted job installs only admitted wheels offline/hash-required and records the actual runtime/compiled modules/loaded providers. Preserve failed acquisition/status; do not upgrade missing versions.
- [ ] Use actual `ParseBAM.annotate_junction`, not ParseSAM. Freeze deterministic informative-single, informative-paired, case-alias-only, no-N and all-short SAM/BED text before outcomes; build/sort their BAMs only with admitted Linux pysam and retain record/CIGAR inventories. Collect every N event, accepted site/support/class and exact output order.
- [ ] The two informative cases jointly cover all three classes, separately annotated sites whose pair is new, duplicate/opposite-strand annotation sites, repeated/multiple N events and mates, flags0x704/supplementary, MAPQ29/30/255, N49/50/51 and mixed S/I/D/H/P/=/X. Include two exact header contigs differing by case, plus a source-only annotation case alias that native input validation will explicitly reject.
- [ ] Run the five core cases with `-q30 -m50 --skip-plot --skip-bed --skip-interact`; one additional informative-single call uses only `--skip-plot` to retain ordinary R/BED/Interact side effects. Six source invocations, no R execution. No-N expects actual exit0/empty primary file; all-short expects actual exit1/header/partial files. Retain these named anomalies; unexpected source failures or different behavior fail qualification rather than becoming generic accepted exceptions.
- [ ] Capture full row keys/order/support/classes, stdout/stderr event/distinct counters, raw status and every file. Pin before-outcome fixture intent, not invented biological results. Fix source runtime capacity before dispatch: four CPUs,8GiB cgroup/no extra swap, no RLIMIT_AS;2GiB retained/3GiB streamed TAR plus8GiB headroom,90min job, bounded per-call deadlines. Source failure/OOM is never a performance win.
- [ ] Reuse existing child-process and private_artifact prepare_tool/finalization_space/encrypt producer interfaces. Copy complete controller import closure and accepted source authority. Always retain available failures; upload only ciphertext and allowlisted transport receipt. The existing public recipient is sufficient; CI never receives the private identity. Do not add runtime Git-process checks when exact pinned body/source authority already supplies the contract.
- [ ] Run the SAME ONE preparation check GREEN under the external guard, shell syntax/diff checks and one self-review. Freeze the four-file delivery and full report externally. Root performs one whole-change review, commits/pushes, verifies exact-head Control, dispatches, recovers authenticated originals and adopts the actual source witness.

### Task 2: Integrate the qualified junction workflow privately

**Files:** new owner `src/metrics/junctions.rs`; modify `src/{cli,report,output}.rs`, `src/metrics/mod.rs`, the existing `tests/workflow.rs`, README/MERGE_RECORD only as needed. This task starts only after actual source adoption; no production scaffold before it.

**Interfaces:** private JunctionClass/JunctionPolicy/SpliceSites/JunctionRow/JunctionCounts/JunctionResult/JunctionAccumulator. `SpliceSites::new(&Annotation, references, profile)`; `observe(&RawRecord, &[(u8,u32)])`; `finish()` returns ordered supported rows and event/distinct totals. Exact source/canonical policies below are binding; ordinary internal type details remain implementation choices.

- [ ] Implement one private original-record accumulator after current validation and before `mem::take` into pileup/source_queries. Reuse interned contigs, site sets and HashMap key-to-row-index; O(annotation sites + distinct junctions), no per-N chromosome clones or second BAM scan. Keep canonical full reference-consuming M/D/N/=/X; source projection ignores=/X and not S.
- [ ] Preserve strict Annotation/exact header presence for both profiles. Junction-only pooling/display follows the pinned source where inputs are admitted; do not relax old TranscriptIndex globally. Both known endpoints classify annotated even if never paired in a transcript; no strand partition. Total N events precede minimum-intron filtering; support counts per-record occurrences including mates; exclude0x704 and MAPQ below30, retain supplementary.
- [ ] Add focused junctions and report-only typed --metrics; default remains the accepted five via Selection::standard. Positive --min-intron defaults50; existing --mapq30 applies explicitly. Keep mandatory shared help/ThreadArgs/output/error layer and truthful inflate-worker semantics.
- [ ] Add junctions.tsv/junctions-summary.tsv plus optional report result/manifest policies to the existing atomic output set. Complete empty/filtered inputs return zero-result headers/counters as the documented source repair. No placeholder or native export/plot/saturation flag.
- [ ] Extend the existing workflow test in ONE connected wave with captured full rows/counters/order, canonical CIGAR difference, alias rejection, empty/filtered cases, focused/report projection,1/4-worker identity and atomic protection. Existing five tests/goldens run normally. Guarded fmt/strictClippy/full debug/release, one whole-change review, then an exact frozen four-native candidate with the new source receipt; no guessed native run or public repository creation.

Asset disposition: donor classifier/count identities are direct private type/algorithm seeds; KnownSites/scan_reads/fetch_introns refactor into the current model/stream with full detail and explicit profile. Existing donor BAM/BED/goldens are test assets only and do not qualify5.0.5. Discard the duplicate reader/CLI/permissive parser, silent invalid-ID skips, summary-only text plumbing, per-event strings, narration and unsupported historical license/clean-room claims. Attribute actual5.0.5 sources/metadata; no presumption of GPL contamination in team-owned donor code.

### Task 3: Measure the unchanged complete new junction workload

**Consumes:** actual Task1 source and Task2 four-native receipts plus new source/lock/ELF identities, never the old five-metric binary or timings.

- [ ] Reuse the complete accepted paired.bam437662663B SHA38174082a8f36ae76a6852b9b970fcb334ab5c8186c133993a0fd652d1d147cd and genes.bed12 9018094B SHA5132086afd5b33bb4d5b2724ef5c4e8275aaba7854a7ec5af0d1902171b88ada from run40592936910. Reverify Linux full bodies:9,810,963 records/58,384 prepared annotation rows, including62 duplicate-name rows; retain the61,177-original/2,793-absent-reference exclusion ledger.338,587 input N operations is not an inferred accepted-junction count.
- [ ] Source primary uses --skip-plot --skip-bed --skip-interact; native emits its full declared detail/summary. Qualification round + excluded warmup + five alternating measured rounds, each source/native1/native4,21 junction commands. Startup/annotation/BAM scan/exports are timed; acquisition/build/hash comparison are not. Source/native1 get one allowed CPU, native4 four; library threads1.
- [ ] Require all full rows/order/support/classes and event/distinct totals equal the actual source. Freeze material serial acceptance before outcomes: disjoint native/source wall ranges and median wall ratio<=.8, or disjoint process-RSS ranges and median RSS ratio<=.75 with median wall ratio<=1.05. Record all CPU/I/O/faults/RSS distributions/losses and native4 scaling, no exact aggregate/cold-cache claim or source-failure win. Preflight actual full retention/time budgets, do not shrink the data/graph after outcomes.
- [ ] Optionally compare one untimed six-metric native report to the retained old five expectations plus fresh junction evidence, without rerunning the old upstream five. Never add old five timing to new junction timing and call that a measured six-metric workflow. Publication remains separate, earned only by current source/API/platform/performance review and live registry authority.

## Preflight and progress

Task1 produces the source receipt consumed by Tasks2/3; Task2 produces the exact native candidate consumed by Task3. These dependencies prohibit parallel implementation of source-dependent production. Source preparation is independent of active Index/Popgen/Ecology/Signal/PLINK work. One controller owns all four new source files to avoid guessed schema seams.

Root adopts this plan under existing unattended authority. Task1 is complete:
actual41277337904 source-only receipt
`.autopilot/oracles/rnaseq-qc-junctions-2026-11-01.json` is accepted after full
authenticated recovery and independent six-call reduction. Task2 is locally
accepted after one whole-feature review and root full release verification;
clean owner is a618e6ab293e2fb7a113d949197e1bb43ecc7733. Its exact additive
four-native candidate is being prepared. Task3 and new four-native/performance/
publication acceptance remain unearned. The
earlier failed startup run remains failed; no old-five proof is relabelled.
