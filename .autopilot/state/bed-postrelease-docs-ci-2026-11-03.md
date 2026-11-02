# BED post-release documentation CI

Documentation-only owner head `26ab4898a31dee2de9cbe317e447417ab35db9a5`
has terminal attempt1 failure in run41509775158. Linux x86_64, Linux aarch64
and macOS x86_64 succeed. macOS ARM job125498296810 has zero steps and
runner_id0. Root reads raw annotations confirming the hosted runner was not
acquired, with ARM capacity notices. This is not an observed product failure.

Small original run/attempt/jobs/job/check/annotations custody is retained at
`/Volumes/KIOXIA/Developments/tmp/bed-docs-ci-failure-20261103.NqSAnV`.
Root independently verifies every PINS.sha256 entry. Manifest SHA-256 is
`a8f1c08c0b74ef9b1fcb3b723ff801ddd7481d09d858f53d3262e52be748441d`.

Root authorizes one failed-job-only rerun at this unchanged docs head. The
three successful original jobs remain explicit attempt1 evidence; any ARM
success must be attributed to its actual new attempt, not mixed into an
invented whole-attempt receipt. No third automatic retry is authorized.

Immutable v0.3.0 remains at bc40084 with its already adopted coherent
four-native attempt2, source package, public assets and installation evidence.
This documentation CI does not replace release assets or benchmark evidence.

## Terminal result

The once-only rerun succeeds at the unchanged exact head. Actual new ARM job
125516665779 runs on runner1000009944 from2026-11-02T20:00:18Z through
20:04:37Z. Root independently reads terminal run and job APIs: format, strict
Clippy, debug/release, normalized package verification, packaged oracle tests
and installed lifecycle all succeed. Non-Linux dependency setup and ARM
benchmark smoke are intentionally skipped, not missing required test steps.
The other three target successes retain attempt1 execution timestamps even
though GitHub's attempt2 jobs API assigns carried-forward job IDs.

Small terminal original packet:
`/Volumes/KIOXIA/Developments/tmp/bed-docs-ci-attempt2-20261103.HHO5xo`.
Root independently verifies all pinned originals; PINS.sha256 SHA-256
`fa6ef411518346fad58db1bdd2f718fa161f1e34fca6d8a04e627ef4fd21d1d3`.
No artifact download, new release, registry retry or performance rerun occurs.
