# K-mer consumer and boundary recheck

Status: source review complete for the two existing consumers. Product tests
were not rerun locally because the physical boot-storage gate remains closed.
The short-sequence finding below is unresolved, not a verified repair.

## Identities and evidence scope

The three local worktrees were clean during inspection. Their exact-head
GitHub Actions results were re-read; all four native target jobs succeeded.
Those old runs do not cover the newly identified missing regression.

| Repository | Inspected head | Successful exact-head CI |
|---|---|---|
| `rsomics-kmer` | `d89e2df0d8eae38b64eb7b43a41f57436fc25bb4` | `30734719265` |
| `rsomics-seq` | `d9734e51c4ed557f6d8790d97a686717ebc4769e` | `31377799041` |
| `rsomics-sketch` | `823fabe3ca092d6e0b30c235cfa961d80c45a543` | `30735400718` |

The retained August release dossier records the published versions and archive
checksums. This recheck does not refresh crates.io state or reproduce the old
performance distributions.

## Actual consumers

Every locally present accepted product's manifest and production source was
searched for the k-mer dependency and imports. Only sequence and sketch have
direct call sites:

| Product | Production call site | Consumed contract | Consumer-side evidence |
|---|---|---|---|
| `rsomics-seq` | `src/operations/kmers.rs::count_kmers` | `KmerCounts::try_new`, checked counting, `decode`; exact two-bit windows with `k` in 1–32 | `tests/kmers_oracle.rs` compares full CLI rows with an independent byte-window implementation; CLI tests cover invalid `k` and frozen tables |
| `rsomics-sketch` | `src/sketch.rs::build` | `CanonicalMurmur64::try_new` and `hashes`; arbitrary nonzero `k`, full-width seed, strand/case canonicalization and explicit invalid windows | `tests/oracle.rs` compares complete signature bytes with sourmash 4.9.4 for `k` 1/17/31/51 and a seed above u32; product tests cover ambiguity and output preservation |

The foundation now has two real product consumers. The old TODO asking to add
a second consumer is stale and can be closed. This does not establish two
consumers for each public API: sequence does not use `CanonicalMurmur64`, and
sketch does not use the exact-count accumulator. Metagenomics has no current
k-mer import; its future minimizer contract does not yet pin a matching hash,
seed, byte order, ambiguity policy and consumer test.

Keep the accepted foundation and existing versioned interfaces while their
maintenance work proceeds. Do not invent a second call site, add a public
crate, or force an unsuitable hash into metagenomics to justify the abstraction.
Before expanding the hashing API, supply two concrete product contracts for
the new item or keep it product-private. Any later internalization of existing
public items needs a compatible migration plan; nothing was removed here.

## P0: short input reaches an out-of-bounds slice

At the inspected k-mer head, `CanonicalMurmur64Hashes::next` returns exhaustion
only when `start > sequence.len().saturating_sub(k)`. For `start = 0`,
`sequence = b"ACG"`, and `k = 31`, that condition is `0 > 0`, so execution
continues to `sequence[..31]` on a three-byte slice. Empty input has the same
path. The iterator's `size_hint` reports zero, but calling `next` reaches an
out-of-bounds slice instead of returning `None`.

The sketch builder calls this iterator on each parsed sequence without a
length filter, so ordinary short FASTA/FASTQ records reach the affected path.
This is not an error in the exact-count sequence operation, which uses a
different iterator. It is a source-proven failure path; no runtime panic or
upstream differential was executed in this recheck.

The existing canonical-hash tests cover exact-length windows, longer inputs,
case/strand normalization, ambiguity, long `k`, scratch reuse and allocation
failure. They omit `len < k`. The sourmash differential uses a 20,000-base
generated sequence for its `k` 1/17/31/51 cases, so it also misses this boundary.
The green CI results therefore do not contradict the source finding.

## Next repair gate

Before the next k-mer or sketch release:

1. Run a failing foundation regression for empty input and lengths `k-1`,
   `k`, `k+1`, including `k = 1` and 51. Check `len`/`size_hint`, repeated
   exhaustion, and reuse of the same hasher from short to valid input.
2. Correct the exhaustion boundary without changing valid-window hashes or
   allocating per record. No public API addition is required for this fix.
3. Add a sketch CLI regression mixing short and valid FASTA/FASTQ records and
   an all-short input. Match the pinned sourmash output and error contract;
   compare complete signatures, not only their count.
4. Rerun both consumer suites, pinned sourmash compatibility, four-native
   exact-head CI, and a no-regression throughput/allocation check. Keep the
   consumer's minimum dependency version and lockfile aligned with the fixed
   registry release before calling the product repair delivered.

The physical-storage restriction is unchanged. No product source, dependency,
registry version, or repository setting was modified by this review.
