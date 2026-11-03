# BED coordinate-generation contract

Coherent flank/slop/shift wave inside rsomics-bed, not new crates. Owner is
clean49cd59e3fb815af81cbf580185a3c88f4ead6610. Existing Genome, BedReader,
coordinate-preserving record output and common/help CLI plumbing are the
intended reuse points; no public foundation or interval index is needed.

Root reads official tool documentation and all three actual operation bodies
from BEDTools2.31.1 commit705ccfdf2c9a77d71560c8adcece0663c2f5e18e.
Complete retained archive SHA-256
1db4e57c1899aede13ef516c61c9b5819cf2790994a556dbb28972488f9946e6.
Sources are behavioral references, not Rust implementation text to translate.
Official overview documentation:
[slop](https://bedtools.readthedocs.io/en/latest/content/tools/slop.html),
[flank](https://bedtools.readthedocs.io/en/latest/content/tools/flank.html),
[shift](https://bedtools.readthedocs.io/en/latest/content/tools/shift.html).
These pages display2.31.0; pinned2.31.1 source and actual binary observations
own the precise compatibility contract. Root independently rehashes the full
archive, three operation bodies, actual tool and helper after capture.

Historical flank30824143, slop5fc4c997 and shift06ae5853 are fixture/reference
assets, not production merges. Their parsers/CLI, missing options, fractional
rules and clipping behavior are not the target contract. The broad routing
category2 means rewrite the execution core and reuse useful assets, not revive
micro-crates. Existing optional/skipping/sorted-oracle harnesses are discarded.

## Once-only integrated witness admission

Root writes and reads the complete helper/literal BED7/genome fixture in
external bed-coordinate-witness-20261103.Yme2eIoj. Admit exactly eight source
calls from one shared nine-record input: three flank, three slop, two shift.
They combine strand/unknown strand, fractions, negative shrink/crossing,
positive/origin points, clipping, input order and opaque fields/headers.
Do not prefill expected output, sort output or omit failures.

Existing read-only Homebrew BEDTools2.31.1 is exact1691808B SHA-256
fc994e1fcfdce8ce8591630d71f21fe67e2275c2a8d967dc0e3dfb4cfa4677a3.
Reuse its already retained version evidence, not another metadata call.
Each command has10s timeout, zero core and32KiB OS file limits per stdout/
stderr; exclusive raw captures and final ORACLE retain statuses/partials.
Packet16MiB plus independent8GiB free headroom, fresh USB external disk12s1
and28GiB free. Use owner guarded-cargo.sh --exec with external resolved Cargo/
Rustup/target/tmp/cache routes, offline metadata and Python bytecode disabled.
No dependency download/build, native implementation, timing claim, retry,
publication or expansion to a malformed-input/overflow matrix is admitted.

## Actual integrated source observations

The first actual guarded capture exits0 after all eight calls; every source
returncode is0, stderr empty, no timeout/output limit/retry. Root reads every
complete stdout and independently rehashes both inputs and all16 captured
streams. ORACLE.json SHA-256
7a149f16fbf83d87d633637fa7ccfd212382d4844d54fffc5434255baf9a306e;
helper8cd46c816e7de32c2c0334e96e8a809e4eb50e62d7073882c42eb5fd950515b6;
inputb02c9fa9ea1d0fdbdd7537de1fce3e63160a54e25b672c4db8db119d3f146c9d;
genomedf776af089d17b5d375fa1976a46154e7b990dc8d19b3148f9d5bcd77642535c.
These are source observations, not native correctness or performance gates.

Normal rows confirm binary32 fractional/truncation rules, strand-relative
asymmetric widths, negative shrink with crossed-coordinate swapping, clipped
one-base shift outputs, genomic left/right flank order, headers and unchanged
opaque suffixes. Slop treats unknown strand as forward; shift uses plus shift;
strand-relative flank omits unknown strands. These are different source
selection contracts, not a reason to blindly reuse strict required_strand.

Point transformations use the source's expanded point span and later report
adjustments. Actual unstranded flank of origin0..0 emits0..-1; negative-strand
fractional flank of15..15 emits13..13 and17..16. Root preserves all original
bytes and asks asynchronously whether native should repair these into bounded
valid flanks (empty clipped flanks omitted), or explicitly reject the affected
point cases. No reply or broader bug-policy authority is presumed. Stop that
affected flank policy/interface decision; ordinary transforms and slop/shift
contract work may continue. Do not advertise unfinished behavior or expose a
bug-emulation placeholder.
