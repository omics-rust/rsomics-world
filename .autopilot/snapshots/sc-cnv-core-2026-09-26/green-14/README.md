# Private creation trial: harness-free lint repair

Green-13 run36730544730 passed actual trial contracts but failed strict
Linux Clippy on test-only module imports/static when the harness-free bench
was linted. Preserve the original source and evidence as an unaccepted run.

Only benches/ingestion_support/mod.rs changes: its test-only imports and NEXT
counter move inside the existing writer-error test. Production helper bytes
before cfg(test), all assertions/counts, trial/timer, Cargo/lock, src/ and all
other148 source hashes remain byte-identical to green-13.
New helper SHA256: 53f665f6e038954ef9b6599d877178f72ba91d6065ffde39bc1cfd9f07b9f2b5

149 exact regular sources, sorted complete snapshot checksums, no macOS xattrs.
Require fresh four-native19Linux/14mac trial tests,55/11/72 prior debug/release
gates, both benches and unchanged strict Clippy before acceptance. No lint
suppression, fake factory, canonical performance or publication follows here.
