# Perf-exempt tools

User-authorized (2026-05-23): tools with **no standard CLI upstream** to perfgate
against may skip the `>1.0× vs named upstream` perfgate. This is only for tools
that genuinely have no canonical CLI to measure against — NOT an escape hatch for
tools whose upstream merely needs installing (those get installed on the 4090 and
gated normally) or for mis-classified Layer-A primitives.

**DONE criteria for a perf-exempt tool:** `compat.rs` (self-consistency / golden /
round-trip / invariant correctness, since there is no upstream to diff against) +
committed & pushed + crate CI green + an entry in this registry. The
perfgate-vs-named-upstream requirement does not apply.

| tool | domain | why exempt | correctness verified by |
|---|---|---|---|
