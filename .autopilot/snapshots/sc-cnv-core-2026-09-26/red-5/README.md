# Runtime-provenance regression tests: red-5

This test-first snapshot adds four pure environment-formatting tests to
green-6: absent versus present UTF-8, non-UTF-8 rejection, and tab/newline/CR
rejection. The proposed format_env_value helper is deliberately absent, so
the named measurement test must fail to compile before the fix is implemented.

Production, fixture and dependency-lock bytes remain unchanged. This run
uses the already reviewed lock with --locked; no resolution exception applies.
The prior green-6 native run is intermediate evidence, not final acceptance,
because its runtime provenance misreports invalid environment values.

No performance result, product CLI or publication is implied. Preserve all
prior snapshots, failed logs and partial artifacts.
