# Index bounded bulk-query optimization

Root reads the complete query/reader path, FAI record validation and the
actual read-only timing diagnosis5acf94821b5348cb90958dead879b47b52e73fdde3658fa80f2092ab0edfe1a8
at index-bulk-readonly-diagnosis-20261101.6VQXN7. Baseline owner is clean
7f2d4fdeda9d0062d10941258b40eee16fdb9042; full reference run41266649396
is successful, with all original custody under separate root adoption.

Five actual plain bulk-1 medians are .228232s native versus .112046s source,
CPU .186245/.063607s and system .141983/.053547s. BGZF bulk-1 medians are
1.081271/.975124s. The throughput gate fails; the scoped complete-chromosome
memory route passes separately. Do not hide those regressions behind the
pipeline sum, change gates or narrow the product to a chromosome-only use.

One product-private implementation wave may remove two evident costs:

- For one worker, execute the same bounded tasks/chunk reader/ordered formatter
  directly rather than roundtrip through an OS worker and two-task barriers.
  Reuse the current planner/emitter instead of another query implementation.
- Limit short plain-reference reads to the requested physical FAI span rather
  than filling a256KiB speculative buffer after each seek. A bounded Read::take
  view and shared generic base/terminator decoder is one suitable route.
  Use natural request/buffer sizes, not dataset/chromosome special cases.
  Keep the existing buffered long-chunk and BGZF behavior; no speculative
  BGZF-member cache/decoder rewrite is included.

Retain1MiB base chunks, finite per-worker storage, ASCII/layout validation,
overflow/error propagation, exact request order/duplicates, per-record default
FAI widths including MT70, explicit wrap/unwrapped output and partial tails.
Keep common's1MiB output buffer, atomic replacement and file/directory sync.
No whole-chromosome allocation, new public API/foundation/dependency or CLI
contract is authorized. Only query.rs and the existing reference lifecycle
test may change; source goldens, owner metadata and other files stay intact.

The measured regression is the real performance baseline, not a fabricated
functional-test RED. Reuse complete existing correctness/CLI fixtures with
one coherent lifecycle extension only if needed; do not add helper-test or
profiling ladders. Run guarded fmt/strict Clippy/full debug/release tests and
retain evidence. Root performs one whole-change review, then owns Git/CI.
Four-native correctness and the unchanged full168-call reference benchmark
must qualify the new exact source before performance/publication claims.
No measured I/O/syscall attribution or future speedup is inferred from static
code; this wave removes concrete overhead and lets the fixed benchmark decide.
