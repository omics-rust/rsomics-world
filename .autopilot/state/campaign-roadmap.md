# Campaign resume entry point

The May 22 instructions below are superseded. They are historical evidence,
not an executable work queue or authorization to run a scheduler.

Resume from these tracked sources, in order:

1. [AGENTS.md](../../AGENTS.md): architecture, storage rules and stop conditions.
2. [TODO.md](../../TODO.md): current executable checklist.
3. [ROADMAP.md](../../ROADMAP.md): product-family consolidation order.
4. The affected product dossier and its specific checkpoint below.

Do not use submodules, workspace-wide Cargo builds, operation-sized public
crates or the old keep-alive instructions. Products are independent repositories
under `/Volumes/KIOXIA/Documents/omics-rust/`. Revalidate local changes, exact
Git heads, CI, registry identities and storage before resuming an action.
Historical release evidence is not proof that a current candidate is ready.

Open checkpoints:

- [K-mer short-window repair](kmer-short-window-2026-09-09.md): correctness
  verified through two consumers; performance interpretation and delivery open.
- [Index release gate](index-0.1-release-gate-2026-08-20.md): publication and
  retained benchmark evidence must be rechecked before release.
- [VCF index selection](vcf-index-selection-2026-09-08.md) and
  [concat audit](vcf-concat-audit-2026-08-20.md): preserve inherited uncommitted
  implementation; do not stage the whole worktree or claim it is validated.
- [Table progress](table-progress-2026-08-20.md): verify the owning repository
  and remaining product gates before implementation or publication.

These checkpoints are entry points, not a reduction of the full portfolio
goal. If an affected action is blocked, record its gate and advance an
unblocked task in the current checklist.

## Archived May 22 snapshot

Everything in this block predates the product-family reset. Its counts,
commands, priorities and claimed automation state must not guide current work.

```text
# rsomics-world campaign roadmap (durable state)

Last updated: 2026-05-22. This file is the cross-session resume point.
A fresh session reads CLAUDE.md + this file to continue the campaign.

## Architecture state (done)
- 98 crates (18 foundation + 80 tools) across 11 domains.
- Monorepo split into per-crate repos under `omics-rust/`; main repo holds
  submodule pointers + `[patch.crates-io]` so `cargo build --workspace` works.
- All CI green (main + per-crate repos).

## Submodule edit workflow (IMPORTANT — crates are separate repos now)
1. `cd crates/<...>/rsomics-<crate>` (it's on `main` branch)
2. edit → commit → `git push origin main` (to the crate's own repo)
3. `cd` back to main repo → `git add <submodule-path>` → commit → push
   (updates the pointer)

## Work order (DO NOT reorder — user-confirmed 2026-05-22)
**Step 2** — deepen/verify existing 80 tools. Then **Step 3** — expand
formats (samtools/bcftools/bedtools missing ops). Then **Step 4** — build
out thin domains (epigenomics/metagenomics/phylo/etc.) to formats-level
completeness via upstream survey → partition → real crates.

## Verification gap driving Step 2
- compat.rs: 54/80 tools have it → 26 missing.
- benches: only 10/80 have criterion benches → 70 missing.
- Perf contract: NO crate ships without compat + bench + perfgate >1.0×.
  70 tools currently cannot prove they beat upstream. This is the Step-2 core.

## Verified perf so far (4090, all >1.0×)
- fastq-stats 9.5× vs fastp
- fasta-utils: grep 18×, revcomp 3.5×, head 3.2×, count 1.85× vs seqkit
- KNOWN FAILURE: bbduk 0.75× vs upstream → needs 4090 flamegraph (blocked).

## Task list = active campaign state (#14–#26)
Step2 perf/compat by domain: #14 epigenomics, #15 fasta, #16 fastq,
#17 bam, #18 vcf, #19 bed+gff, #20 genomics, #21 remaining domains.
Step3: #22 samtools, #23 bcftools+bedtools.
Step4: #24 epigenomics buildout, #25 metagenomics, #26 remaining+new domains.

## Keep-alive
- Session cron heartbeat every 15 min (`<<autonomous-loop>>`), job 62588dd3.
- This file + git + memory = cross-session-death resume.
```
