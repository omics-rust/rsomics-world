# Private PLINK maintained SHA-2 backend candidate green-16

Complete committed source from clean private owner main
223e342f845bdb280bd369395483f8f72a1d445f, tree
f0596e2476ebe2c53d61cc2c70d4c2e002c3f445. All 307 Git bodies and committed
modes are retained. Relative to green-15 owner
270caf52d2aa87375cfd1ab5c6ef8d0ca2acf29c, 304 are unchanged; only Cargo.toml,
Cargo.lock and src/io/transaction.rs change. No application sidecars or scratch
extras are included.

The existing SHA-256 dependency is pinned to sha2 0.11.1 with default features
disabled. Only its required lock closure changes. The necessary Digest output
API adaptation formats each finalized byte into one 64-capacity String using
standard lowercase zero-padded hexadecimal. The 64 KiB incremental reader,
I/O propagation, digest values, manifest schema and file/directory durability
contracts are unchanged. No custom crypto, forced CPU feature, hidden worker,
public API or new foundation is introduced.

The exact-source native gate expects 17 library and 5 integration test
executions per debug/release mode, zero failed or ignored and zero main/doc
tests. Four metadata tests appear in both harnesses; these are not 22 distinct
scenarios. The unchanged scientific graph retains six lanes, 36 generations,
74878 raw-cell checks, 123 reports per worker profile and 66 worker records.

Earlier negative performance evidence and green15 observations remain
unchanged. This snapshot does not establish new native-four or same-file source
qualification, throughput/RSS advantage, performance acceptance, release
readiness or publication permission. Those gates must consume this exact
newly frozen source separately.

Archive members are sorted exact Git blobs and committed modes from ls-tree,
regular USTAR only, uid/gid/mtime 0, empty owner/group names and gzip mtime 0.
No directory, .remember, scratch, AppleDouble or private-cohort entries occur.
Capture performs no dispatch, product execution, dependency download,
publication or gate waiver.
