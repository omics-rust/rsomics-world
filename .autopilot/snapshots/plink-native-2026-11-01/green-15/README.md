# Private PLINK table-arena compaction candidate green-15

Complete committed source from clean private owner main
270caf52d2aa87375cfd1ab5c6ef8d0ca2acf29c, tree
0e3392246fda32bc7a569906118832c077221202. All 307 Git bodies and modes
are retained. Relative to green-14 owner 7cee8e4366ef957981c1741055c3ff716dd5b53b,
305 are unchanged; only src/dataset/metadata.rs and
src/dataset/metadata/text.rs change. No hook or scratch extras are included.

Completed PSAM/PVAR tables shrink their String and field-range Vec capacity
after width validation and before Arc sharing. Transient BED/VCF builders,
types, layout, validation, UTF-8, field ranges and zero-sentinel behavior stay
unchanged. This is private table-construction work, not a new API or foundation.

The exact-source native gate expects 17 library and 5 integration test
executions per debug/release mode, zero failed or ignored and zero main/doc
tests. Four metadata tests appear in both harnesses; these are not 22 distinct
scenarios. The unchanged scientific graph retains six lanes, 123 principal
reports per worker profile, 66 worker records and 74878 raw-cell checks.

No dependency, version, C++ ABI, codec, hashing, thread policy, output schema,
generation SHA, atomic recovery or durability semantics change. Earlier
negative performance evidence remains unchanged. This snapshot does not
establish native-four or same-file source qualification, throughput, RSS,
performance acceptance, release readiness or publication permission. Those
gates must consume this exact newly frozen source separately.

Archive members are sorted exact Git blobs/modes, regular USTAR only, with
uid/gid/mtime 0 and empty owner/group names, gzip mtime 0 and no directory,
scratch, AppleDouble or private-cohort entries. Capture performs no workflow
dispatch, product execution, dependency download, publication or gate waiver.
