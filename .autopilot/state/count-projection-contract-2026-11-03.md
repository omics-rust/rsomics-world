# Count read-position projection frontier

This is source inventory plus one actual upstream endpoint witness, not an
implemented/advertised native CLI or a native compatibility result. The
original-CIGAR fraction correction precedes it.

Pinned Subread2.1.1 source and bundled guide are retained in
`/Volumes/KIOXIA/Developments/tmp/count-projection-20261103.i3Q509A7`.
`readSummary.c:3161–3218` and guide950–953 apply5prime extension, then3prime
extension, then reduction, independently for each mapped mate. Direction
uses the raw SAM reverse flag, not the second-mate-adjusted counting strand.
Only outer assignment sections change. Reduction chooses the already
extended outer endpoint and collapses the read to one base. Original split
classification and original denominator intervals remain unchanged.

Left extension clips at one-based coordinate1 using the original read
position. Right extension clips at the alignment-header chromosome length,
not an annotation maximum. Forward5prime extends left; reverse5prime extends
right. Source CLI help mistakenly describes3prime as upstream; the actual
code and guide1122–1124 establish downstream behavior.

Extensions accept0 through2147483647; default0. Documented read2pos values
are3 or5, default disabled. Source also accepts undocumented suffixes starting
with3/5; an exact typed3|5 native option would reject those malformed values.
No new public foundation is needed. Header lengths must reach the private
projection seam; preserve original reference identity and extent.

Source preserves zero-length assignment sections whereas native Blocks
currently retain only positive spans. This is a concrete endpoint-contract
gap, not a proven upstream bug or a reason to ask for a bug policy.
Do not silently project a different endpoint or label this compatibility
complete. Shift is not added by this inventory; it is outside the current
dossier's read-projection slice.

Use one mixed forward/reverse/spliced/deletion/boundary SAM/BAM workflow and
one paired raw-strand/asymmetric-extension/reduction/fraction workflow for
actual oracle comparison. Avoid a per-helper matrix. No installation,
download, owner edit, build, or test was performed by the bounded inventory.

Root subsequently executes three official source calls on5D10M at one-based
position101, LN1000. `read2pos 5` selects101, `read2pos 3` selects115, and
extension5=10 followed by reduction5 selects91. Each exits0 and counts exactly
its expected single-base target. This confirms the leading deletion's empty
first assignment section affects the endpoint: selecting first positive
Block start106 would be incompatible. Preserve endpoint/section metadata at
the parse seam rather than deriving projection solely from positive Blocks.
Other terminal patterns are not inferred from this one witness.

Original argv, stdout/stderr, matrices/summaries and fixture bodies remain in
the original source packet as `projection-leading-gap-*` and
`PROJECTION-LEADING-GAP.json`, SHA-256
`0c6b9ed937d8dbbc074c81ddd8b900846effa54d243ebb01c5f117a75bb1c0d7`.
No native projection has been implemented or tested.
