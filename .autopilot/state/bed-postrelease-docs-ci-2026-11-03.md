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
