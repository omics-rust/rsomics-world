# Private PLINK locality-preserving bounded batches candidate green-10

Complete committed source from clean private owner main
9d43df417bf2a729438fb4194e9f82cbba10fe6c, tree
989afd93e6114727ef9546c8349e4ceb25be174c. Exactly three source files differ
from green-9 owner80fafbf8: src/make/pipeline.rs, src/make/mod.rs and
src/ld/pairs.rs. All305 committed bodies and Git modes are retained.

Make separates packed/optional-track execution density from unchanged
conservative memory admission, amortizing ordered handoff over reusable
batches. Pair computation ranges are coarsened independently of formatted
output size; existing bounded65,536-byte output blocks, recycling, weighted
segments, deterministic ordering and serial fallback remain.

Dependencies, public API/CLI/schema, numerical policy, PGEN Rust/C++ adapter,
generation hashing, validation predicates and all file/directory fsync
contracts are unchanged. No new scheduling framework or foundation is added.
The unresolved default-atomic/--durable policy remains unresolved.

Both focused regressions were observed failing before implementation.
Local full debug/release each passed eight library tests and one complete
six-lane CLI graph, with strict Clippy and formatting passing. Root read the
entire three-file delta, independently verified identities and ran the full
all-feature suite (nine tests passed); one fresh-context whole-wave review
found no material blocker.

Review limitation: the actual fixture does not cross the new16K candidate-pair
split inside a job; the four-million-pair planner test covers that splitting
and contiguous coverage. The root-reviewed representative diagnostic must
cover the execution boundary before performance or release acceptance.
It has not been executed. This is not native-four or performance acceptance.

Archive members are sorted exact Git blobs/modes, regular USTAR only, with
uid/gid/mtime0 and empty owner/group names, gzipmtime0 and no directory,
scratch, AppleDouble or private-cohort entries. Existing green-9 and all its
authenticated negative results remain untouched; no speed/scaling/RSS claim,
gate waiver, publication, workflow dispatch or retry is authorized here.
