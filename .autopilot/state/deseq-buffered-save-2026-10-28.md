# Buffered fitted-bundle save

Owning clean localmainc9f99b70c191c6ed29d42c8d4cd49ed151b573b7 changes only
the import and save sink in src/artifact.rs. It consumes existing published
common0.12.3 write_output(Some(path),...) instead of unbuffered write_atomic.
The helper buffers1MiB, explicitly propagates flush errors, and retains the
same atomic transaction. Named '-' rejection and JSON error classification
remain unchanged. No new public API/dependency/schema/math/JSON bytes change.

The real whole-workflow baseline40605899832 located the cost in the initial
read/fit/save command, not reload. Full original pasilla native old/new A/B
then ran one warmup and three timed pairs at1/4 workers. Mac medians/ranges:
old1 8.26[7.85,8.47]s -> new1 0.41[0.39,0.41]s;
old4 7.57[6.75,7.92]s -> new4 0.22[0.22,0.22]s. System CPU falls from
7.51/7.05s median to0.01s; no syscall count or lower-RSS claim is made.

All16 fitted bundles are exact-byte equal within worker count, all full states
match except workers, and original source/reload/four-export checks pass with
unchanged bounds. This is a local native A/B diagnostic, not a new formal
upstream or scaling acceptance. Parent freshly reruns all12 release tests,
strict release all-target Clippy and Rust1.91 formatting successfully, and
reviews the exact two-line diff plus actual published helper implementation.
Guards verify external paths, Cargo metadata target and APFS below77.27%.

Report and verification:
/Volumes/KIOXIA/Developments/tmp/deseq-buffered-save-20261028.rjDl8a/REPORT.md,
verification.json. Old binary
16f0ed3d8c2a8d2442c16d7d17b5c9ab024a824dc8f48de6bf78d2592adc3a70;
newf19563fc1cac02080cf7cbe91002d9d280d0af8318eedeeaa838376b64887d19.
Cargo.lock remains501544e3615498b7f26c06bb5ea36c4d027b61f7366cce4a11b6efcbfc82e241.

Green5 freezes the same36-file schema4 inventory; archive
f72b71c96bfed743ce13756ce6265f7745d8dadf1be2bacc8a8ee68d0c5ea8c8,
manifestb75297f3621a8e18864daee09005fec0814dd8dd013f60cadd6b2951ebaddc1e.
Existing exact-byte snapshot guard and two compact native-wrapper/Bash tests
pass. Evidence scratch: /Volumes/KIOXIA/Developments/tmp/deseq-green5-check.tY5vju/.
Controller40611403553 is SUCCESS at exact
42f378fe633649b7234c5cedaf424d0d4ab62da5. Unchanged four-native complete-workflow
run40611588271 at that exact head against green5 is SUCCESS on all four
platforms. Original APIs/ZIPs/logs/source/binaries and20 whole-workflow gates
are retained/verified; source diff to green4 is exactly the two save lines.
Receipt:
/Volumes/Zane's HDD/rsomics-fixtures/evidence/deseq-native-2026-10-28/run-40611588271/verification.json,
SHAc86fec10a22f4b69a6470b99562f0bdc8d4de12496e488c227a69d962b767bf4.

The existing performance harness now pins that accepted Linux artifact
12782868065,325788430bytes, ZIP
037b9c76ecda311280ecadaebccce244da78e2490be28bf366c1bcd775d9ba9b,
binary630431e494e5fcba34eb175c7cfe4a68ff071afd249e29619e2385ab7c061059.
Its36-file/schema4/20-workflow admission passes on the real retained artifact.
Three compact tests/Bash syntax pass; parent reads the full scoped diff and
confirms source driver, inputs, flags, bounds, trials, timing and decision
remain unchanged. Full495 control tests pass with two existing skips.
Controller40614101382 is SUCCESS at exact
12d615548c52092a7f325e8138cd75e85a1f9e0e. Fresh performance40614312003 is
dispatched at that head against the accepted green5 executable and is now
SUCCESS, with original2900-file artifact retained/verified. Fresh R-inner
median5.495s versus native1/4 1.05/.80s passes strict throughput (5.23x/6.87x),
strict process RSS and1/4 scaling (1.3125x). No I/O advantage. Actual host is
EPYC7763, not the older VM; no old timing is reused. Authoritative receipt
SHA31c6da8517f44c26ac827be5b153e29f3778e270954bfb7eb348fbe06329436e,
under evidence/deseq-real-workflow-performance-2026-10-28/run-40614312003/.
See deseq-real-workflow-performance-2026-10-28.md. Do not reattribute this
C9/schema4 result to concurrent schema5/local changes or rerun unchanged fits.
Old green3 performance/green4 native evidence remain immutable. Local trend
coverage and publication remain separate gates; continue the full portfolio.
