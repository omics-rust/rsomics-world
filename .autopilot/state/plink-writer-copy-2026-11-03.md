# PLINK writer track-copy cleanup

Base private da64da53b22532131fc536def25997fcd3f9de49 has the exact qualified
green15 tree. SHA candidate evaluation is closed with its negative result.

Root reads make's serial and pipelined consumers, Rust owned read/append
interfaces and native Tracks/read/append code. Tracks::copy validates dimensions,
clears six complete mask arrays and five counters, then unconditionally
overwrites all six masks and five counters. Remove only that writer-side
reset call. Keep reader-side reset, owned buffers, every copy/check, codec
dispatch, exception propagation and transaction durability unchanged.

For929 samples the source expresses720 redundant zeroed bytes per variant;
compiler elimination and actual traffic/time are not established. This is
redundant-work cleanup, not a measured speedup or a new public API. Do not
start another four-native/benchmark campaign for this individual deletion.
Retain the existing whole six-lane/worker1+4 raw-state consumers and complete
local fmt/strict Clippy/debug/release verification on physically external paths,
offline, without new helper tests, fabricated RED or dependency movement.
These bounded checks do not substitute for a future actual release gate.

Only native/pgen_shim.cc may change in the owner. Preserve all historical
negative measurements and existing comments; add no narrative comment,
foundation, borrowed-pointer shortcut, cached-count invariant or CLI feature.

## Private cleanup integrated

Owner ef896240e79389563ed233997601d4d03e2517a4 deletes exactly one reset call;
tree71855db9f81662cd50a73e98e5ceb9a75ca02cd1, worktree clean. Root reviews
the exact diff and full overwritten fields; reader reset remains. Native body
SHA-2560fbe5e916e03bb091dd5a1992766d8df22f839b9920db09f53be6150aa843038.

Actual handle3276 exits0 after all four guarded commands: fmt, strict all-target/
all-feature Clippy, full debug and full release. Each profile passes17 library
plus5 workflow executions, zero failed/ignored, including the complete six-lane
worker1/4 graph checking74878 raw cells. Root reads complete debug/release
logs. Four metadata cases occur in both harnesses; not22 distinct scenarios.
No test/input/dependency/threshold/comment changes, added tests or downloads.

Logs are in external plink-writer-copy-20261103.ffqPlv4a:
format8a42b2da0e14ad1183f2783900166ce189bd8ef22e099dd8bf703518c79c57d9;
Clippy7a81ca466c7165459affccf7352b0ca51164c5389e8cbf83dc980bfc5d3df654;
debug2666b2d79bdbcbdbf887445cc1f4790a29c347ea56fe809f9841bc6545030540;
release8909a032f7897d6a4b3196f89a698cbb60109ef170183f6bcd448223ce8bf98a.
Each guard verifies actual USB external disk12s1, resolved Cargo target and all
Cargo/Rustup/tool caches/tmp. Independent free-space precheck exceeds10GiB;
actual external free space remains about29GiB. No boot project output.

This is local correctness/cleanup acceptance only. No throughput/RSS benefit,
four-native qualification, installed-artifact acceptance or publication is
claimed for ef89624. Further work should use coherent product/release gates,
not reopen a separate experiment/review ladder for this individual deletion.
