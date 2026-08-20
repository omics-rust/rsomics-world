# rsomics-table 0.1.0 release performance — 2026-08-20

Product revision: `2bd0fd3698c152bb27e7d0d7635d51fb41655112`

Exact-head GitHub Actions run `32314817480` passed native Linux and macOS on
`x86_64` and `aarch64`, the pinned full-oracle job, package verification, and
the Ubuntu 22 benchmark-artifact build. The measured binary SHA-256 was
`6e490331da90475cd2eaa84a7e3fc0cd77af381a8cc049bf70261547ad549f34`.

## Protocol

The release run used an Intel Xeon Gold 6238R host running Ubuntu 22.04 and
Linux 6.8. CPUs 32–35 were pinned to four separate physical cores on one NUMA
node. Each benchmark used three warmups and ten randomized paired runs. The
selected CPUs were 96.7–100% idle during preflight and no active swapping was
observed. Raw evidence remains at
`/data5/rsomics-table-bench-20260820-r3/results-release-2bd0fd3`.

The pinned oracles were csvtk 0.37.0, GNU datamash 1.9, and BEDTools 2.31.1.
Seven paired outputs were byte-identical. BEDTools prints large sums with fewer
significant digits, so `groupby --consecutive` used the tracked semantic
comparator: exact rows, fields, groups, and counts, with relative tolerance
`1e-9` only for sum and mean. Its binary SHA-256 was
`505ec5f3c4f6e6d400a2cfb85474047a518a949636e3286e00f8a21727e83af6`.

## Results

Times are arithmetic mean ± sample standard deviation in seconds. Peak RSS is
from a separate successful GNU time run over the same command and fixture.
The ratio is upstream time divided by `rsomics-table` time; values above one
favor `rsomics-table`.

| Workload | `rsomics-table` | Upstream | Ratio | RSS KiB, ours / upstream | Release decision |
|---|---:|---:|---:|---:|---|
| `validate`, 5M CSV rows | 1.335714 ± 0.012765 | none | — | 3,584 / — | bounded standalone validation |
| `validate`, 5M gzip CSV rows | 1.838960 ± 0.020278 | none | — | 3,584 / — | bounded compressed validation |
| `select`, 5M CSV rows | 1.683502 ± 0.026859 | 2.271596 ± 0.011946, csvtk | 1.349 | 4,480 / 23,248 | throughput and memory pass |
| `select`, 5M gzip CSV rows | 2.192043 ± 0.025500 | 2.186690 ± 0.004806, csvtk | 0.998 | 4,480 / 29,568 | equivalent throughput; 84.8% lower RSS |
| `filter`, 5M CSV rows | 1.821590 ± 0.014456 | 20.095507 ± 0.026634, csvtk | 11.032 | 4,480 / 23,188 | throughput and memory pass |
| numeric `sort`, 60M rows | 146.778690 ± 1.280696 | 161.397980 ± 2.770950, csvtk | 1.100 | 26,253,696 / 14,442,876 | 10/10 paired wins; 81.8% higher RSS disclosed |
| inner `join` | 3.789779 ± 0.035742 | 5.057810 ± 0.058655, csvtk | 1.335 | 445,140 / 1,387,008 | throughput and memory pass |
| global `groupby`, low cardinality | 1.982616 ± 0.021071 | 2.449727 ± 0.032311, datamash | 1.236 | 3,584 / 710,528 | throughput and memory pass |
| global `groupby`, 500k groups | 3.704217 ± 0.059613 | 2.765885 ± 0.014374, datamash | 0.747 | 186,352 / 710,528 | 33.9% slower; 73.8% lower RSS |
| consecutive `groupby`, 5M rows | 1.580043 ± 0.028275 | 1.620464 ± 0.016415, BEDTools | 1.026 | 3,584 / 32,256 | throughput and memory pass |

The release is accepted with two explicit trade-offs. Numeric sort favors
throughput while using more memory, and high-cardinality global grouping
favors memory while losing throughput. The gzip selection path is effectively
tied in time and retains a strict resource-use advantage. These are supported
release behaviors, not universal claims that every operation is faster.

## Integrity audit

- revision and source dirtiness: exact revision, zero dirty files;
- paired tables: eight files, ten complete pairs each;
- validation: two files, ten successful runs each;
- command records: 18; output files and recorded hashes: 16 each;
- timing JSON records: 160; GNU time records: 18;
- exit statuses: all zero; stderr files: all empty;
- output hashes: all 16 recalculated successfully;
- comparisons: seven byte comparisons and one semantic comparison replayed;
- benchmark binary hashes: matched the exact-head CI artifacts.

The earlier `r2` run is preserved as debugging evidence. It predates the final
record-buffer reuse and semantic BEDTools comparator and is not release input.
