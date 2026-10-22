# Reference-only oracle locale failure

Actual failed run39781233583/attempt1, source b3691d50b50c884f4753390d6db07ce7bfd79ea7:
canonical capture/validation passed,17 new checker tests passed0.408s, then
new Rscript stopped `cannot enforce C locale`, exit1 at locale initialization.
No probe object/input RDS or grouped/pooled call was reached; no edge outcome
may be inferred. Original artifact12427039794,1576253509bytes,
SHA256c91bc2d13f74a7ecdcd3e943dd9d2bc5e7958fb0e5829273a6ef5547f24739d0
is being retained with original APIs/job logs/ZIP. Do not overwrite/retry it.

Root inspected actual original job119832723431 log and primary R4.6.1 source.
Official archive https://cran.r-project.org/src/base/R-4/R-4.6.1.tar.gz,
40926649bytes,SHA2564da6e61d2c0aac5f14a2e7e432cb5fcc269efe83da4293050ba7f03dff4e2cf4,
VERSION4.6.1,SVN90187(2026-06-24), retained externally under
rsomics-fixtures/sources/r-4.6.1-filter-2026-10-22/.
src/main/platform.c2239-2255 shows R's LC_ALL branch sets only selected
categories; LC_MESSAGES/PAPER/MEASUREMENT have separate branches2302-2318.
Canonical exporter495-499 explicitly resets those three; the new script176
does not. This identifies a source-backed initialization omission; the exact
live before/after category values were not printed by the failed launch.

Next bounded TDD: factor the unchanged old initializer into a sourceable
private function; add actual R4.6.1 regression that seeds those categories
non-C, logs before/after and requires all-C/idempotence. Run early after
setup-r, before costly dependencies, and authenticate the intended old-code
RED. Only then add the missing category resets and rerun the full oracle.
No C-locale requirement is weakened. Canonical exporter/36case/152fixture/
accepted receipts/pin heredocs remain unchanged. Boot gate still forbids local
R/build/install; this is live native CI evidence, not a Python mock claim.

RED preparation committed8042d110: old initializer is factored unchanged,
new source guard permits only test sourcing, and actual R regression seeds
the three omitted categories non-C, logs all eight before/after, requires
complete C locale and idempotence. Nineteen focused Python tests pass1.319s.
Root adds an early same-opt-in workflow step after setup-r/before dependencies,
retaining exact script/test copies, hashes and real R stdout. Actual native
RED remains pending; no category fix has been made.

Fresh independent regression/sourceability/workflow review approves with no
Critical/Important finding. Coherent controller suite423tests48.979s passes,
two existing skips; architecture/whitespace pass. Native early-step RED will
be dispatched only after exact-head controller CI; a non-C setup failure or
parser error does not count as the intended initializer failure.
