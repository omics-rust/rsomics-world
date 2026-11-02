# Sequence strand-extraction representative gate

Scope: reviewed source3ba1818e85477573995f6a391522f2ef35054c79, not the
historical0.2.0 release's timing. Execute only after actual native4 CI passes.
This gate does not complete the remaining sequence dossier operations.

## Fixed useful workload

Use the complete restored SRR341550 R1 input,87439836 compressed bytes,
SHA256 d7a15c1762d64a5434ced0cc665d7f5d167ca81a71e239f8237b9cd490dd7683,
at `/Volumes/Zane's HDD/rsomics-fixtures/fastq/seq-strand-20261102/SRR341550_1.fastq.gz`.
Freshly validate its complete record population and uncompressed byte size;
do not substitute a small subset for the full input.

Compare these three workflows, preserving complete named raw outputs:

| Workflow | Native | Source SeqKit2.14.0 |
|---|---|---|
| regional FASTQ extraction | subseq -r 21:80 | subseq -j1 -w0 -r21:80 |
| reverse complement | revcomp | seq -j1 -w0 -r -p |
| connected extraction→reverse complement | subseq -r21:80 piped to revcomp | corresponding two source commands piped |

Use CLI stdout redirected to a fresh output file for both tools, not native
named-output fsync versus source buffered stdout. Pipeline commands use
pipefail and retain every process exit. Both pipeline stages must succeed;
a successful final command cannot hide a failed upstream stage. No /dev/null
timings, omitted records, quality-field-only comparisons or source NaN masks.

## Measurement and retention

- Freeze clean source/lock, freshly build locked/offline release under the
  inspected external guard, and bind native/source executable hashes/version.
- Keep source single-thread budget1 and native's real serial contract; no fake
  native threads flag or claimed scaling. Record machine/OS/CPU/RAM, storage
  and all flags/environment. Pin CPU only if the host actually supports it.
- One warmup per tool/workflow, then five alternating native/source pairs
  per workflow:36 workflow invocations,48 executable invocations including
  the two-stage pipelines. Retain all exits, complete output files, hashes,
  wall/CPU distributions and raw platform resource records.
- Admit external output capacity before starting: reserve enough for36 times
  a validated full-output upper bound plus8GiB independent headroom. The
  current HDD has about72GiB free, not an unlimited budget. If insufficient,
  choose another verified external output volume; do not delete originals or
  silently drop trial retention. Never put output/cache/tmp on the boot disk.
- Compare complete bytes for every native/source output and confirm expected
  full record count and matching quality lengths. Qualification is outside
  timed windows; actual timed commands still perform their normal work.
- Report individual-command process peak RSS with correct platform units.
  Pipeline child rusage is not exact simultaneous tree memory; no aggregate
  pipeline-memory or universal RSS claim follows. CPU and I/O provenance are
  evidence, not throughput substitutes.

Each operation needs a strict throughput or process-resource advantage under
the established portfolio rule. The connected workflow must remain useful
and correct; record wall losses and overlapping ranges, not only a favorable
median or isolated kernel win. No tolerance/threshold tuning after results.

Retain any failed trial and the input/source/binary/output provenance. One
representative campaign on the qualified candidate, not a campaign after
every helper edit. A failure changes the next optimization decision; it does
not authorize an unchanged retry. Version bump, immutable GitHub delivery,
public install and registry publication are subsequent separate gates.
