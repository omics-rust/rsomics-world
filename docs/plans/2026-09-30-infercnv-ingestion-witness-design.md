# inferCNV raw-input witness design

## Intent and boundary

The next inferCNV slice establishes real-package evidence for the three-file
`CreateInfercnvObject` boundary before adding a parser to the unpublished
`rsomics-sc` product. It does not alter the accepted prepared-input step-14
core, advertise a CLI, publish a crate, or claim full inferCNV compatibility.
The user delegated routine design and execution decisions; source or evidence
contradictions still stop the affected compatibility claim.

Use infercnv 1.28.0 from commit
`b421d9405c97a309b081ef86d455e976df93eae4`, installed by the existing
manually dispatched oracle workflow. Keep package identity, source archive
hash, R session and input hashes with the output. The workflow uses runner-temp
storage; local Cargo/R builds remain prohibited while boot APFS exceeds 80%.

## Alternatives considered

1. Directly merge retired `rsomics-infercnv`: rejected; it parses a GTF-like
   dialect, substitutes zero on malformed numbers and has no genuine R oracle.
2. Implement a general R `read.table` clone first: rejected; its quoting,
   comment, NA, gzip, duplicate and decimal behavior needs observed witnesses.
3. Extend the installed-package oracle with targeted creation-only witnesses:
   selected. It resolves the format and model contract before product code.

## Witnesses and outputs

The witness runner consumes an existing, independently validated synthetic
oracle bundle and, in shipped mode, the fixed shipped bundle. For each bundle,
it calls `CreateInfercnvObject` separately on the canonical plain count TSV and
on a gzip encoding of the same bytes, with the bundle's gene positions,
annotations, ordered reference names and creation options. It asserts exact
expression, gene-order, cell-order, count-data and ordered reference and
observation index-map equality between branches and against the existing
stage-1 RDS. It records both source and gzip hashes, package/session identity,
all creation arguments, exact matrix dimensions, ordered map names/indices,
and the matched RDS path and hash. The original noncanonical shipped gzip is
not silently treated as equivalent to the canonical TSV.

A small separate positive witness uses shuffled annotations, an unannotated
extra count column, a position-only gene, a count-only gene, excluded chrX,
chromosome first-appearance rank different from lexical order, reference
request order different from observation name order, and the decimal literal
`0.076439` plus exponent notation. Its input bytes and stage-1 outputs are
retained. The runner records decimal values as 17-digit text and hexadecimal
floating-point text; it does not assert a universal decimal conversion rule.

The independent checker rejects missing files, changed hashes, unpinned
package/source identity, mismatch between plain/gzip/stage-1 output, altered
ordered maps, wrong dimensions or numeric nonfinites. A successful witness is
evidence only for these files and options. The reader's remaining quirks
(duplicates, malformed data, `V1`, count limits, missing reference and random
per-group cap) remain explicit future oracle cases, not supported by inference.

## Ownership and next gate

The product-local raw reader will later build `PreparedCounts` without
weakening its unique-ID, positive-depth, finite-value and coordinate
invariants. Its supported text dialect and decimal policy must be chosen
against this witness, not inferred from the source alone. Raw ingestion remains
inside `rsomics-sc`; `rsomics-help` will own future command presentation, and
no new public foundation is justified. After ingestion, stage-15 `samples`
clustering needs its own real-package oracle; it is not implied by step 14.
