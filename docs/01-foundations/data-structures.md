# Data structures — consumer and upstream survey

Updated 2026-09-09. This is an operation and dependency map, not a queue of
crates to publish. Upstream availability does not establish rsomics completion,
compatibility, performance, or a public-foundation boundary.

## Current ownership

The accepted names remain the [30-product/nine-foundation allowlist](../00-overview/registry-reset-keep.txt).
Detailed behavior and release gates belong to the product dossiers.

| Capability | User workflow and owner | Current boundary | Required contract |
|---|---|---|---|
| Exact DNA k-mer encoding and counting | `rsomics-seq kmers` | Existing `rsomics-kmer::KmerCounts` and codec APIs | Checked k, strand policy, ambiguity, overflow, deterministic count rows |
| Canonical Murmur64 windows | DNA signature construction in `rsomics-sketch` | Existing versioned `rsomics-kmer` API; only sketch currently consumes this hasher | Complete-window exhaustion, alphabet, canonical strand, hash lane, seed width, scratch reuse |
| Persistent MinHash / FracMinHash | `rsomics-sketch` construction, inspection, comparison and search | Product-owned signatures and operations; later collection/index/gather work stays in this product | Hash/selection profile, abundance, interchange, downsampling, score semantics, output transactions |
| Minimizer index and lookup | Planned `rsomics-minimap2 index` / `align` | Embedded minimap2 engine, not a replacement foundation | Pinned engine/preset/index parameters, MMI compatibility, mapping results |
| Taxonomy-labelled minimizer database | Planned `rsomics-metagenomics` database/classification slice | Product-private until a second matching consumer is proven | Minimizer/spaced-seed profile, taxonomy revision, LCA assignment, database integrity, read reports |
| FM/FMD index, BWT and suffix array | Full-text sequence matching | No validated current consumer establishes a new public foundation | Sentinel/alphabet, coordinate width, count/locate semantics, index construction and memory evidence |
| HyperLogLog | Approximate distinct-item estimation | No accepted operation currently justifies extraction; evaluate inside its eventual consumer | Error profile, hash/precision parameters, merge compatibility, serialization |
| Bloom / Cuckoo filters | Approximate membership or candidate filtering | Consumer-local dependency choice, not an independently installable rsomics product | False-positive behavior, capacity, insertion/deletion, exact-verification policy, thread safety |
| Compacted / coloured de Bruijn graph | Graph construction and graph queries | Survey-only; no assembly/graph product is added to the current allowlist | Oriented graph model, compaction, colours, interchange, real graph/query oracle and resource evidence |

Current source anchors are
[sequence counting](https://github.com/omics-rust/rsomics-seq/blob/d9734e51c4ed557f6d8790d97a686717ebc4769e/src/operations/kmers.rs),
[sketch construction](https://github.com/omics-rust/rsomics-sketch/blob/3802b1aaca54a95c14dbdeb6571aff4eb5ebafc3/src/sketch.rs)
and the [foundation consumer recheck](kmer-consumer-review-2026-09-09.md).
Two products using different k-mer APIs do not prove that every public item
has two consumers. Preserve existing compatible interfaces while auditing
them; require two concrete contracts before any public expansion.

## Hashing and sketches

The checked-in foundation wraps `nthash 0.5.1` and implements the selected
MurmurHash3 profile. The upstream [ntHash project](https://github.com/BirolLab/ntHash)
and [SMHasher/MurmurHash3 source](https://github.com/aappleby/smhasher) are
behavior references, not interchangeable hash identities.

A hash name alone is insufficient for persisted data. Pin the variant, lane
width, seed width, byte order, case/ambiguity policy and strand canonicalization.
Do not replace MurmurHash3 with ntHash, xxHash or another hash merely because
an isolated benchmark is faster. Internal hash tables may select a different
hash privately when no external representation or algorithm contract depends
on it; there is no portfolio-wide default established by this survey.

The [sourmash command reference](https://sourmash.readthedocs.io/en/latest/command-line.html)
documents sketch construction, comparison, search and gather as related
workflows. Our pinned compatibility profile and implemented-versus-planned
surface are defined in the [sketch dossier](../10-products/metagenomics-sketch.md#rsomics-sketch),
not inferred from the latest upstream manual. FracMinHash, fixed-size Mash
sketches, exact k-mer counts and taxonomic classification are not synonymous.
Retaining the same upstream function name is not enough to establish file or
score compatibility.

The canonical short-input repair is source-verified but not yet a delivered
registry repair; its [current evidence](kmer-consumer-review-2026-09-09.md)
takes precedence over old completion ticks or historical speed claims.

## Sequence indexes are not one shared index format

[Rust-Bio's documented data structures](https://docs.rs/bio/4.0.1/bio/data_structures/index.html)
include BWT, suffix arrays and FM/FMD indexes. The previous claim that Rust-Bio
lacked an FMD implementation is withdrawn. API availability alone does not
validate a particular aligner, coordinate size or workload.

[Minimap2's algorithm and index guide](https://github.com/lh3/minimap2#algorithm-overview)
describes minimizer extraction and lookup, with index parameters fixed when
the index is built. Its [product dossier](../10-products/minimap2.md) keeps
that engine-owned representation inside the product. Neither FM-index code
nor the sketch hasher is substituted for the embedded minimizer engine.

FAI, BAI, CSI, TBI and GZI access belong to the
[format-index survey](indexing.md) and the relevant products. An FM search
index is not a prerequisite for `rsomics-index` bgzip/tabix, and a sketch
collection index is not a reason to move sketch policy into that product.

## Probabilistic structures

[probabilistic-collections](https://docs.rs/probabilistic-collections/0.7.0/probabilistic_collections/index.html)
documents HyperLogLog and Bloom/Cuckoo families; the
[Cuckoo filter reference](https://github.com/efficient/cuckoofilter) is another
behavior and implementation source. These are candidates to evaluate, not a
ranking of the fastest or most complete libraries.

Choose an implementation only through a product workload. Record accuracy
as well as CPU, memory, I/O, construction, query and merge costs. A filter
must not silently change an exact operation into approximate deduplication;
the operation must either verify candidates exactly or explicitly expose and
validate an approximate profile. Concurrent access, deletion and serialization
are separate contracts, not properties inferred from the algorithm's name.

No blanket SIMD, GPU, parallel scalability or maintenance claims are carried
forward from the old survey. Re-establish them for the selected version and
actual consumer before they influence an implementation or release decision.

## Graph construction

[GGCAT](https://github.com/algbio/ggcat) documents compacted/coloured graph
construction and graph queries, with Rust and C++ APIs. That is a real
workflow reference; it is not evidence that a small adjacency-map helper
implements assembly, graph compaction or its interchangeable formats.

The historical graph source is retained for inspection, not automatically
promoted. The current allowlist has no standalone assembly or pangenome-graph
product. Adding one would require a separate product-boundary decision and
a dossier; this survey does not make that decision. Related upstream scope
remains in the [assembly survey](../02-genomics/assembly.md).

## Historical assets and adoption gates

The [routing ledger](../00-overview/portfolio-inventory.tsv) is a historical
source snapshot, not a current dependency graph. Its inbound counts must not
override current product manifests and call sites.

| Asset | Retained evidence | Disposition for this survey |
|---|---|---|
| `rsomics-fm-index` | Ledger head `3f81d6b52fa870f48ae88742b4f21b24015be807`; snapshot marked dirty; local source includes BWT/SA and occurrence storage | Inspect the owned diff before reuse; adopt or refactor only into a concrete full-text-search consumer, not the bgzip/tabix slice |
| `rsomics-debruijn` | Ledger head `96743c6c519b1cc5d69307d58c2147b120965fdb`; snapshot marked dirty; local source exposes a canonical k-mer adjacency model | Retain as an unvalidated implementation asset; do not call it a compacted/coloured graph product or republish its old name |
| `rsomics-kmer` | Current consumers and repair evidence linked above | Retain the accepted foundation; repair existing contracts without speculative public APIs |

Neither retired asset is deleted, modified or accepted as release-ready here.
The earlier crate/version catalog remains recoverable in Git history.

For any later adoption or extraction:

1. Name the owning product operation and its observable data contract.
2. Compare existing dependencies and team-owned assets against that contract.
3. Pin the actual source, license and attribution requirements; an academic
   algorithm does not establish a software implementation's license.
4. Test malformed input, boundaries and upstream compatibility, then measure
   representative CPU, memory and I/O with retained provenance.
5. Keep product policy internal. Promote a public item only after a second
   named product and consumer-side tests demonstrate the same contract.

No new public crate, product, dependency or completion claim follows merely
from an entry appearing in this survey.
