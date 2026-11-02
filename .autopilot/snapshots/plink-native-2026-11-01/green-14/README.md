# Private PLINK metadata-construction candidate green-14

Complete committed source from clean private owner main
7cee8e4366ef957981c1741055c3ff716dd5b53b, tree
985b13ec27da3ee7fab26053d542bb955eacbaa2. All 307 Git bodies and modes
are retained. Relative to green-13, only src/dataset/metadata.rs and
src/dataset/metadata/text.rs change; no hook or scratch extras are included.

PGEN metadata construction binds first-match column ordinals once, consumes
owning exact-size rows without an intermediate Fields vector, and reserves
bounded typed vectors and REF+ALT capacity. Existing types, compact arenas,
validation order, unknown fields, UTF-8, INFO/PR output and memory accounting
remain the product contracts. This is private implementation, not a new API
or public foundation.

The exact-source native gate expects 17 library and 5 integration test
executions per debug/release mode, zero failed or ignored and zero main/doc
tests. Four metadata tests appear in both harnesses; these are not 22 distinct
scenarios. The unchanged scientific graph retains six lanes, 123 principal
reports per worker profile, 66 worker records and 74878 raw-cell checks.

No dependency, version, C++ ABI, codec, thread policy, output schema,
generation SHA, atomic recovery or durability semantics change. Original
green-13 negative performance evidence remains authoritative. This snapshot
does not establish native-four or same-file source qualification, throughput,
RSS, performance acceptance, release readiness or publication permission.
Those gates must consume this exact newly frozen source separately.

Archive members are sorted exact Git blobs/modes, regular USTAR only, with
uid/gid/mtime 0 and empty owner/group names, gzip mtime 0 and no directory,
scratch, AppleDouble or private-cohort entries. Capture performs no workflow
dispatch, product execution, dependency download, publication or gate waiver.
