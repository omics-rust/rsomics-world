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
Next: exact-head controller and unchanged four-native complete-workflow gate;
then bind a fresh formal same-host benchmark to its accepted Linux executable.
Old green3 performance/green4 native evidence remain immutable. Local trend
coverage and publication remain separate gates; continue the full portfolio.
