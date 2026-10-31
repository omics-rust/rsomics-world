# Signal native candidate with source-compatible empty BigWig

Owning repository: `/Volumes/KIOXIA/Documents/omics-rust/rsomics-signal`.
Owning main commit: `eb6c38327e9aa8ad56a2f3a0b1b0e805194bfbd5`.
This38-file candidate remains private and publish=false, not an admission or
release receipt. Cargo.lock and nonempty track/zoom arithmetic are unchanged.

Source archive SHA256:
`b6ba14f12e8c05a22bb4aa83aa04e9b2457eda290759a0e550f45c34ea4c3466`.
Source manifest SHA256:
`3e97a7f3c1c94528996ce4cf7213a51ea582eb9c61e3271ee56d9003c5017d89`.
Cargo.lock SHA256:
`d2770d56b6295dd5ab10d39f3a42ab3a8ef6d69bdd1c9e079f661a96d53736be`.

Actual green-2 gate41214999273 passes all four native components and the
source reads of a/b/compare/average/12000-run rollover, then fails on empty
minVal. Original raw files additionally prove native131NaNs versus pinned[]
because pyBigWig uses indexOffset!=0 as its hasEntries predicate.

The private empty writer now retains source extrema initializers and a zero
index offset, removing the unused empty index. The private preflight accepts
that representation only with zero covered bases/items and no zooms; readers
return no intervals without asking the codec for a nonexistent tree. The one
chromosome fixture is165B, not the previous217B or pinned writer404B. Byte
length is not a semantic equality requirement; the hosted comparator retains
strict headers, full-values cardinality/classes, intervals and statistics.

Actual existing whole-workflow regression first fails on zero minVal, then
all10 library tests and1 six-operation workflow pass, as do formatting and
strict all-target Clippy. Public empty summaries remain same-shaped NaNs;
no data interval or fabricated zero substitutes for emptiness.

Packet: `/Volumes/KIOXIA/Developments/tmp/signal-empty-header-correction-20261031.YOZSFM/`.
Old snapshots and failed originals remain immutable. Native4 plus remaining
same-file source consumers, representative performance and optional sortedBED
policy remain separate unearned gates.
