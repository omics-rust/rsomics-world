# BED relative-distance source contract

No native implementation or performance acceptance. This operation belongs
inside rsomics-bed; midpoint vectors and reporting policy need no new public
foundation or dependency.

## Independently inspected sources

Root reads complete BEDTools2.31.1 reldist.cpp/reldistMain.cpp, bundled tests
and historical Rust library. Pinned archive is the same complete705ccfdf source
at `bed-makewindows-inventory-2026-11-03.md`, SHA256
1db4e57c1899aede13ef516c61c9b5819cf2790994a556dbb28972488f9946e6.
Fresh member SHA256:

- src/reldist/reldist.cpp:902ff1fd8dcd97004ad6fd15b25b95c3953a704edae7a17d25aab20a79f3691c
- src/reldist/reldistMain.cpp:dc6c93b8facb107f416f8f514bf96b484a613ac1ab9e59311811b9ad415c31a2
- test/reldist/test-reldist.sh:b04d57d306823fa1ddca30653536dc80becb92b767f0fea4a039b2f04dcec742

Historical clean36cdfdead5f6617265d412145df28a272f84236a library SHA256
468f6461fab2c4cc21fb2ba09eddc6a83b9571ec5b0a36032cb3081fdaf48788.
Disposition3: retain corrected lower-bound straddle algorithm and fixtures;
rebuild parsing/CLI on existing checked product plumbing. Old reader silently
discards malformed rows; missing detail mode and false sorted/100-bin/range
documentation must not migrate. Valid relative distance0.50 is in source goldens.

## Whole operation

Only A/B inputs and detail-versus-default-summary are recognizable modes.
Group/sort B midpoints by chromosome, stream A in input order; inputs need not
be sorted. No strand, block or overlap-fraction policy is involved. Lower-bound
straddle accepts exact first/last only with a pair, handles duplicate midpoints
as zero, skips one-B/missing/outside chromosomes. Symmetric point expansion
preserves midpoint and detailed printing restores the original point geometry.
Summary emits only occupied floor(distance*100)/100 bins, header,2-decimal bin
and3-decimal fraction; detail appends3-decimal distance to each eligible A row.

## Three real source calls

Packet `/Volumes/KIOXIA/Developments/tmp/bed-reldist-witness-20261103.wvi3trlA`.
Root reads the full prepared helper/four inputs and independently rehashes them
before one guarded call. Helper SHA256
4d26e643a285e85e5678ca05c7a65d78e27334c3e494724287c4ff1f8ffd0a51.
Physical external disk12s1, external Cargo/Rustup/target/cache/scratch paths and
28GiB free are checked. Existing read-only source executable1691808B SHA256
fc994e1fcfdce8ce8591630d71f21fe67e2275c2a8d967dc0e3dfb4cfa4677a3.
Three calls only: integrated normal summary/detail and isolated high-coordinate
summary. Each10s/32KiB-per-stream/core0, packet8MiB/free8GiB; no retry, native
build, new per-helper matrix or timing. All exit0 without timeout or stderr.
Root reads every full output and independently checks all helper/input/tool/
argv/status/output identities. Normal summary has3 zero and3 half-distance
queries; detail retains six eligible rows, duplicates and their input order.

The differing source casts are an actual defect: valid B coordinates near
1100000000 narrow their endpoint sum through int32, and a mathematically
straddled0.500 A query disappears (header-only output). An async decision is
requested: use full coordinates and record incompatibility, or explicitly reject
affected large coordinates. This affected policy remains held, not implicitly
resolved by map's earlier answer. Ordinary source contract is established.
ORACLE.json SHA256
3e4a03ffcd7221a92acbda48c23fb2a22f922130228a0aea2ea4810a1bc4b126.
