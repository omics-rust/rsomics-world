# Reference-only oracle locale failure

Resolved: actual39789882345 passes the unchanged native regression and both
reference-only calls. See infercnv-reference-only-accepted-2026-10-22.md.
The history below preserves both intended failures and intermediate status.

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

Exact-head controller39785772947 passes72cdbf8fc95d6f6787adaead5fb2ca41fac25518.
Live locale RED39785979929/attempt1 at that head is active, job119849185911;
it intentionally runs the unchanged old initializer. Current progress setup-r,
not a test outcome. Root's working compact-artifact/numeric-machine diagnostic
additions are NOT in that run. Their focused tests first fail absent hook/
artifact, then pass20tests after implementation; no locale category fix yet.

Failed full launch39781233583 is now completely retained:1576253509bytes
matches API SHA,1423 unique safe ZIP members pass CRC. Edge folder has only
six pre-factory source/function/exporter/io files, no witness/input/route RDS
or calls. Original logs ZIP SHA
c0a9443312dbef96bbc9db1e67e78c24aa863c206fd196176836eb03d418382c;
independent external audit also stream-verifies all939 new-run canonical
witness file pins. This separate new canonical output is not substituted for
the accepted Sept30 receipt or152 fixture.

## Authenticated native RED and bounded repair

Run39785979929/attempt1 at72cdbf8fc95d6f6787adaead5fb2ca41fac25518
completed the intended early native RED. Setup and non-C seed preconditions
passed. Actual before/after output shows LC_MESSAGES/PAPER/MEASUREMENT stayed
C.UTF-8 after the old initializer, while the other five categories were C;
the unchanged strict predicate raised cannot enforce C locale, exit1.
Original artifact12429033624,6445bytes,SHA256
a31b04bd79b63b4db12cfbb980d174213d7bbd2731ee19edf2b39de36745011b
matches API metadata; all six safe unique members pass CRC, copied source
and regression match exact head. Original logs ZIP SHA256
8eb4c9ba3bb043de99fb9db220490afaecfac8caf4801104258feb5c0c6ebf4c.
Independent audit and originals remain in external run39785979929/.
Neither failed launch observed a grouped or pooled edge outcome.

Repair3b65144e40ec786ca9b4fd2317997268ae5f63c3 changes only the initializer:
explicitly reset the three canonical categories after LC_ALL, retaining the
strict final all-C assertion. Native regression bytes remain unchanged,
SHA2560958c8a99712622a908a3d0ef15583c6717079ccc5cd0931259d7e478471eb9e.
Fresh independent review approves the repair and root's separate compact
evidence upload/base-R R.version/.Machine diagnostic, with no important
finding. Full raw upload, canonical exporter/I/O helper and literal pins
remain unchanged; embedded workflow remains byte-identical.

Fresh focused20 tests pass1.691s; coherent full424 tests pass52.166s with two
existing native-only skips. Architecture and whitespace checks pass. These
are controller checks, not native locale GREEN or reference-edge acceptance.
APFS boot occupancy81.84% still prohibits local R/Cargo/build/install;
Python -B tests use explicit external cargo home/target/TMPDIR. Push exact
head, require controller CI, then dispatch one new full native oracle.

Exact-head controller39789572225/job119861673079 passes
bf1cd1512e0d2071591a4b52850a21d86cc41a7d:424tests35.784s, OK(skip2),
architecture consistent. Independent original run/jobs/logs are retained in
external controller-39789572225; logs ZIP SHA256
2f44e9dae0f0cc46298940345a4706633387838bc6ffeb850199c47beb41da72.
Root dispatched39789882345/attempt1 at2026-10-22T13:46:47Z, same exact head,
datasetshipped/samples_clusteringtrue/ingestion_contractsfalse, snapshots
blank. Live API authenticates workflow_dispatch and job119862798656.
At this checkpoint setup-r is active; no native GREEN or edge status is
inferred. Independent collector will bind executed selectors from logs and
audit the new compact official artifact without replacing accepted evidence.

Later live job API reports Test samples locale contract success and advances
to system/package dependencies. This is native step GREEN as reported by API,
not yet original-byte acceptance: locale before/after/.Machine and actual edge
outcomes still await completed logs/compact official artifact. No result is
filled from the expected regression, and the failed originals stay retained.

Terminal parent now completed success at14:44:36Z; whole-run logs match the
completed job log. Independent audit and root original-byte/structure checks
admit the bounded reference-only outcomes in the separate accepted record.
The full raw canonical artifact is not downloaded/admitted by this acceptance.
Exact-head controller39795994590 passes d0fec79f:432tests46.290s OK(skip2),
architecture consistent; root retains original APIs/logs ZIP SHA256
04a47bb9b274a95e1a8524ef017833e219ebdb0903a42cd4cef91457788015ef.
