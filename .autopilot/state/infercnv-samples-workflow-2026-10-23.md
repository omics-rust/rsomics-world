# Integrated samples/Ward candidate

Latest: green19/run39819012036 now passes dependency metadata and compiles.
Its canonical CLI test fails at the newly written literal group count:
actual Microglia/Macrophage19 vs mistaken expected23. Earlier assertions
already establish8508 genes/184 cells/6187 selected/2321 outliers. Root reads
the actual accepted maps.tsv and shipped full01.cells.tsv: Microglia19,
Oligodendrocytes23, observations142. Original reference order is Microglia
then Oligodendrocytes, independent of global cell-column order.

Correct only this new test's hand-entered reversed cardinalities, and bind
each expected count to its literal group name. No production behavior,
old oracle fixture, numerical tolerance or frozen comparison is changed.
Green20 differs from19 only in tests/cnv_cli.rs and repeats integrated native
validation. The first full 1/4-worker equality assertion has not yet executed
successfully; do not claim it from the passed earlier assertions.

Actual green19 terminal detail: Linuxx86/Linuxarm/macOSarm debug/release
reach that CLI assertion; original standalone filter/tree totals pass.
macOSx86 instead fails before Rust at oracle download, TCP read timeout.
Whole-log ZIP SHA0bfcb19c4117f4da585f8f95f9aabf88bd5b31d9ce2a0edf0a80ac81d3090537;
original APIs/logs are external under the corresponding run directory.

Current productbasebe35e65; integrated source is working-tree-only until its
native checks pass. One overall static review found no Critical/Important
defect; CLI fixture cleanup minor was fixed. Root standalone Rust1.91
formatting/whitespace checks pass; controller433 tests54.116s passes(skip2).
Exact world25733e8d controller39818105400 succeeds.

Actual green18/run39818103394 fails at cargo metadata --locked before any
compilation/tests/CLI run. Automated source/oracle checks pass. Original
failed logs/APIs are retained externally at evidence/
infercnv-samples-workflow-2026-10-23/run-39818103394/.

Root cause: rsomics-help enables Clap wrap_help. Adding its lock node alone
omitted clap_builder's newly enabled optional terminal_size dependency. The
existing cached official registry record for clap_builder4.6.7 explicitly
maps wrap_help to dep:terminal_size (^0.4.4). Existing seq's lock has the actual
terminal_size0.4.4 registry/checksum node; its rustix/windows-sys dependencies
already exist in this product's graph. This is not a numerical/CLI behavior
failure or evidence against the tested standalone filter.

Repair only the incomplete Cargo.lock graph: add that known node and the
clap_builder link. No version update, new runtime feature, local Cargo
resolution/download, changed test expectation or extra lock-resolver framework.
Freeze green19 and rerun actual four-native integrated validation; the
original failed green18 stays immutable. Runtime and performance remain
unverified until the corresponding commands execute.

Parallel measurement feasibility: SSH4090 has existing Rust/Cargo1.99.1,
R4.6.1 and timing tools, but load~118 on112 logical CPUs with unrelated jobs
makes it unsuitable for clean replacement-speed claims. Read-only inspection
created no scratch or jobs. Its historical /data3 project symlinks resolve
to /data5, which has479GiB free; do not write based on the symlink's apparent
filesystem. Prefer an actual measured, clearly scoped hosted or quiet-host
run after functional validation rather than claiming speed from threading.
