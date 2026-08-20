# rsomics-bed 0.2 performance gate

Date: 2026-08-20

Status: passed for release candidate 0.2.0. Exact-head release CI is green;
registry publication is credential-blocked.

## Evidence identity

- performance source: `d7b1507b178053a087862255d84a244e4921f192`;
- documentation source: `52764bdc401ea3a2512f3deba10453757da32a6f`;
- release candidate: `02b85a1a348c271e485cca629dc4e71fa075388a`;
- release-candidate four-platform CI: run `32329089591`, passed;
- clean local 0.2.0 package SHA-256:
  `6ce9d3cc727e385e4e66accc85c774b18f8ac7b902315c0782551e4266b1c80e`;
- release binary SHA-256:
  `9a55972f96e11bf087515b61799575505291d12d67902b4c965b8b9e32b51ff8`;
- BEDTools: 2.31.1, release archive SHA-256
  `fc7e660c2279b1e008b80aca0165a4a157daf4994d08a533ee925d73ce732b97`;
- BEDTools binary SHA-256:
  `287fb59cd3f68f43e45df5e807596fdc2f170be38c9b8953aaceb511b20cbfb0`;
- base runner SHA-256:
  `ba09fce09fb7152e1a02d983defa282a951d64931b8a9ef4d8de66e64d48f6f2`;
- relation runner SHA-256:
  `9d16cbc2b808c93210b5a3b4c637a08fd2a84b6e33545e1a73b6324a9d004e76`.

Publish workflow run `32329397022` built and verified all 93 package files,
then failed at upload with crates.io `403 authentication failed`. The workflow
was not repeated, and no tag or GitHub release was created. The performance
decision remains valid independently of that credential gate.

The representative host was `dell-Precision-7920-Tower`, Linux
6.8.0-90-generic, `x86_64`, with two Intel Xeon Gold 6238R CPUs. The candidate
was built with Rust 1.95.0; CI independently exercised the declared Rust 1.91
minimum. Measurements were pinned to physical cores 48-51 on one socket after
the selected cores and sibling threads measured 94-100% idle. GNU `time` ran
one warmup and ten measurements. Relation pairs alternated implementation order.
Tables report mean wall time with sample standard deviation and median peak RSS.

## Published-operation regression gate

The original one-million-record fixture and complete output hashes were
unchanged from the 0.1 gate.

| Operation | rsomics-bed | BEDTools 2.31.1 | Speedup | RSS KiB, ours / oracle | Decision |
|---|---:|---:|---:|---:|---|
| sort | 0.398 +/- 0.004 s | 1.097 +/- 0.011 s | 2.76x | 209,664 / 360,490 | pass |
| merge | 0.204 +/- 0.005 s | 0.226 +/- 0.005 s | 1.11x | 2,688 / 4,480 | pass, narrow |
| intersect | 0.699 +/- 0.006 s | 2.698 +/- 0.026 s | 3.86x | 99,678 / 344,960 | pass |
| subtract | 0.602 +/- 0.008 s | 3.076 +/- 0.022 s | 5.11x | 19,712 / 344,960 | pass |
| complement | 0.243 +/- 0.005 s | 0.308 +/- 0.004 s | 1.27x | 5,376 / 4,480 | throughput pass |

Raw JSON:
`/data3/liangjy/rsomics-linux-x86_64-20260820/bed-gate/results/base-1m-d7b1507.json`,
SHA-256
`b4fb912d945dd83d1467f40f39ecf534cb8f27641cf8b312b3d12c290c3a0e8e`.

## Relation gate

The relation fixture contains one million A and cluster records across ten
chromosomes. Window B has one true match per A plus a duplicate every fiftieth
record. Closest B has equal-distance left and right records plus an overlap
every fiftieth record. The dense count lane contains 5,000 mutually overlapping
A and B records. Every complete output matched the pinned executable byte for
byte before timing.

| Operation | rsomics-bed | BEDTools 2.31.1 | Wall / CPU speedup | RSS KiB, ours / oracle | Decision |
|---|---:|---:|---:|---:|---|
| cluster | 0.229 +/- 0.006 s | 0.932 +/- 0.010 s | 4.07x / 4.15x | 2,688 / 4,480 | pass |
| cluster same-strand | 0.291 +/- 0.003 s | 1.509 +/- 0.018 s | 5.19x / 5.33x | 18,816 / 451,372 | pass |
| window pairs | 0.676 +/- 0.010 s | 2.820 +/- 0.024 s | 4.17x / 4.19x | 158,908 / 470,400 | pass |
| closest | 1.349 +/- 0.011 s | 1.662 +/- 0.026 s | 1.23x / 1.23x | 316,696 / 28,636 | throughput pass |
| closest unsigned distance | 1.420 +/- 0.019 s | 1.927 +/- 0.025 s | 1.36x / 1.36x | 316,704 / 13,136 | throughput pass |
| dense window count | 0.157 +/- 0.005 s | 2.194 +/- 0.020 s | 13.98x / 14.39x | 3,584 / 8,064 | pass |

Input identities:

| Input | Bytes | SHA-256 |
|---|---:|---|
| cluster BED6 | 33,433,300 | `108c4923808666f7a8942a1699f6f3fffff91c32a211d6e11cd9cceb9b705333` |
| relation A BED6 | 34,877,780 | `dcd79d7e23a118d4d2033d6325a4f9dbc672c87168948f364bdf4fe5d807bc60` |
| closest B BED6 | 72,473,070 | `b56cdd6494a76c497df84bcaf63a93a3617178b2b39730519f452175b37acedc` |
| window B BED6 | 35,575,290 | `0ec499733c02882c82aafaf04a970af6e4e1eb393907b800fd498fff298bd9a5` |
| dense A | 113,890 | `6f238adc556453f0fae3540a06e3f2fee21d38f9d28bfb6fd21ccaca2b60e556` |
| dense B | 107,780 | `726c8bf6160d2a41bf3bea165bf6817b307af0a4b6139fa42ad534c0c97982c5` |

Output identities:

| Operation | Lines | Bytes | SHA-256 |
|---|---:|---:|---|
| cluster | 1,000,000 | 39,877,775 | `2fdc8c4160e24dcc0349df758ed9a320419383b6b084f9f6d453c6c7748e2101` |
| cluster same-strand | 1,000,000 | 40,322,196 | `b7189c6902ecf6c8a6d76c5ce6027d46025c2411acf2ce12b52228be170872b4` |
| window pairs | 1,020,000 | 71,150,580 | `c8b9ce3929dd859573089a9e198facdffd91bdf49fe25182a81a9c6ef9d58453` |
| closest | 1,980,000 | 140,096,100 | `1f2f82e843e97431220560c6d1a0f4e161203a1569afb27b92144fc46d2fc82f` |
| closest unsigned distance | 1,980,000 | 146,016,100 | `199b21ac7d3abfc22881ab7f8e525ea12e43be5a2e81569534c34d4d96f55429` |
| dense window count | 5,000 | 138,890 | `6cd3bf156bf32c431e95cfdba6bf308ec1228cbf8b92cccaea47d9d11354eb85` |

Raw JSON:
`/data3/liangjy/rsomics-linux-x86_64-20260820/bed-gate/results/relations-1m-d7b1507.json`,
SHA-256
`695c0d7a34020a006251d18e212d8760a1b53a4e59b7c07855ee594c7d8eee14`.

## Rejected candidate and trade-offs

Head `c8e09eeaaba0aa5ce3c20d7f0089cb85c8380264` failed default closest at
0.943x, with 1.766 seconds and 577,304 KiB RSS. It was rejected. The accepted
compact relation-index refactor keeps B in one raw buffer, removes duplicated
virtual coordinates, and reuses the index start order. On the same fixture this
moved closest to 1.349 seconds and 316,696 KiB.

Closest still uses more memory than BEDTools because it accepts arbitrarily
ordered B and preserves stable B-file tie order; the upstream executable uses
coordinated sorted streams. The release therefore claims closest throughput,
not memory superiority. `window` also guarantees stable B-file hit order rather
than reproducing the UCSC-bin traversal order exposed by BEDTools when multiple
hits cross internal bin boundaries. Membership and multiplicity remain
compatible.

Rejected raw JSON:
`/data3/liangjy/rsomics-linux-x86_64-20260820/bed-gate/results/relations-1m.json`,
SHA-256
`f66f1248f30e084b1bb78c51c6147ef636097dcf21f41d311cba4800f4486a6a`.

No public foundation was added. The compact record store and relation index
encode BED parsing, ordering, tie, and output policy and therefore remain
private to the product until a second concrete consumer proves a policy-free
shared contract.
