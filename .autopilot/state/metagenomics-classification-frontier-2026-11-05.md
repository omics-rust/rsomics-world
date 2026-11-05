# Metagenomics classification source frontier

## Product and source boundary

The next domain slice remains the dossier's complete nucleotide database
build/inspect/classify/standard-report lifecycle inside `rsomics-metagenomics`.
No new crate, empty module, public placeholder, or foundation API is created.
Owning main is clean at `34e60b53de0ba1f5937e28fd4f68abb4a1442435`;
the published0.1 amplicon lifecycle is unchanged.

The bounded read-only inventory confirms these historical source assets:

- taxonomy `996982d4fead6e8ed8a4ff738d0f00d67c8fcd7e`: refactor/internalize;
- report `bf1e47d687ff3fe7bd0d87535819ffcacd0bf22b`: parser/fixture seeds only;
- tax-assign `b611492cd6f1a8535991f3fbb4178300a1b0ecdc`: discard production;
  retain negative fixtures. Exact-kmer votes are not a Kraken classifier.

## Exact source oracle established

Official release: https://github.com/DerrickWood/kraken2/releases/tag/v2.17.1

Commit `5e2aa928d00b96d61f204d517437637863da1d8c` is independently resolved
from the official tag API. Codeload archive SHA-256
`8cf3eeca9078d40a5d07af00dbc87a56a0c1cc11d09ee36442388a0a710052e9`.
All103 archive members are bounded, unique, safe regular files/directories.
All98 original file bodies match the extracted source after compilation;
installed scripts differ only by documented install-path/version substitution.
No local wrapper repair exists. A reviewer initially inferred one from an
incomplete read, then retracted it; direct archive/source/install comparison
independently disproves the inference.

Packet:
`/Volumes/KIOXIA/Developments/tmp/metagenomics-kraken-source-20261105.aSYNz0Mi`.
Read the complete installer/Makefile before building. Actual install is only
the packet's `oracle/`, using existing Apple Clang and read-only Homebrew
libomp with explicit include/link flags. Guarded actual external disks, free
space, compiler/module/cache/scratch paths precede download/build/workflows.
No dependency/software installation or source deletion occurred. Upstream
warnings and linker deployment-version warnings remain in `build.log`;
the current27.2 host executes the resulting2.17.1 oracle successfully.

Read pinned manual custom-database sections, taxonomy loader, minimizer
scanner, classifier assignment, compact table and option serialization.
Kraken is MIT licensed; no upstream license exception is assumed.

## Actual upstream lifecycle witness

`witness.py` SHA-256
`e6d53132fe9a7d495fd4c377a71740c7fb7bb2db04bb861cef47a1ffbc9718b2`.
Successful `witness-attempt3` retains full commands/stdout/stderr, taxonomy,
references, reads, real database, inspect, classifications and reports.
Fixture is project-owned synthetic sequence/taxonomy data, not biological
accuracy evidence: seven named taxa, three800-base references, eleven reads.
Fixed seed `0x5E2AA928`, k35/l31/seven spaces, explicit no masking, build1worker.

Observed eight classified/three unclassified reads include leaf assignments,
shared-minimizer genus LCA, balanced sibling and cross-genus LCA, lowercase,
reverse complement, ambiguity, shorter-than-k and absent sequence. Full1/4
worker classification and six-column report bytes agree. The separate
eight-column minimizer report retains the same hierarchy/count fields.
Inspect reports700 occupied cells/capacity13165/key29/value3bits.

Two earlier fixture assertions failed and are retained, not accepted native
comparisons: directly concatenated equal-length fragments do not imply equal
minimizer support; inserting ambiguity only between fragments still gives
different start/reset contexts. The final balanced cases put both fragments
after the same ambiguity context and actually show67/67 support. Do not turn
equal raw lengths into an assumed vote equality or label this as an upstream
defect without further evidence.

`WITNESS.json` SHA-256
`4fe48c204248fc61427d00ac3613a8bdeaec3b2e080960213743ee598573d805`;
tracked copy `.autopilot/oracles/metagenomics-kraken-upstream-2026-11-05.json`.
Source verification receipt SHA-256
`45967fce188478d1ee3d256daff779a820ac3cb0c1f77a9d142dac00979dbced`.
These prove the upstream baseline only. No native classifier, performance,
four-native classification acceptance or new release exists.

## Representation ruling and next complete implementation

Ruling: preserve upstream compact-table semantics inside an explicitly native
versioned container, not an exact full-key default. Compact lookup accepts a
truncated fingerprint; collisions also merge build values through LCA
(`src/compact_hash.cc:109-193`). Replacing it with a normal exact map changes
assignments and reports. Fixed Murmur finalizer, linear probing, capacity,
pruned internal taxonomy numbering, spaced/toggle masks and deterministic
reference/block/zone insertion order are part of this compatibility profile.
This stays private to this product. If wrong, database identity and output
parity would require a format revision; do not publish before direct witnesses.

Use fixed-width little-endian fields, section lengths/checksums and normalized
decoded options in the native format. Upstream writes native `size_t` and raw
`IndexOptions` struct padding, so raw options bytes are not a portable or
deterministic format contract. Do not claim native/Kraken `.k2d` interchange.
Exact full-key lookup can be a private testing oracle, not the default policy.

The controlled compact-collision witness below now supplements the sparse
baseline; the latter's5.32% occupancy/29fingerprint bits alone do not qualify
collision behavior.
Capacity estimation and its8192 adjustment affect probe paths and must not be
silently replaced by exact-distinct sizing. Eight-column distinct-minimizer
reports use HLL upstream; this tiny agreement does not justify exact counting
for a later minimizer-data profile. Then implement the complete bounded
build/inspect/classify/standard-report lifecycle with existing common/help/
seqio; require retained outputs, representative resource evidence, one full
review and four-native exact-head CI before exposing a stable release.

## Controlled collision and partition witness

`collision_witness.py` SHA-256
`c11b6526c0d60fee61e1b14737bec9b337c877066eeda284d3cff7d10de9730f`;
successful packet `compact-collision-partitioned/`. This uses actual upstream
`build_db` with explicit capacity1024, block256/subblock64 and reserved taxon
width3 or28. It is an intentional stress profile, not the wrapper's default
capacity estimator or representative error-rate measurement. No fake database
or Rust classifier is involved.

The same three references yield702 stored entries with29fingerprint bits,
versus662 with four fingerprint bits. One hundred new fixed-seed reads are
checked by the actual upstream minimizer scanner: all3843 query keys are absent
from the full702-key partition-aware reference set. The29-bit profile leaves
all100 unclassified; the four-bit profile classifies99. Cross-genus reference
key intersection is zero; direct domain-level minimizers increase from0 to24,
demonstrating collision-induced LCA changes at build time.

Both profiles' actual1/4-worker builds have identical table/taxonomy bytes and
decoded options, and actual1/4-worker classification/report outputs agree.
Raw options padding is excluded from the semantic comparison. The classifier
result is not a native correctness or performance gate.

A first retained diagnostic used a whole-record reference scan and found700
keys, while actual partitioned construction stores702. The strengthened
`scanner_partition_witness.cc` invokes the exact source scanner with the
builder's actual intervals and accounts for all702. This confirms that block
and subblock parameters affect collected keys as well as insertion order;
do not silently change them for performance. The earlier diagnostic and its
original `COLLISION-WITNESS.json` remain retained, not relabeled as the stronger
partition-aware proof.

`COLLISION-WITNESS-PARTITIONED.json` SHA-256
`80a79e1d679c8912974c2983705b07fbf5d5fa6f34b3230678ade0c39ad033ea`;
tracked copy `.autopilot/oracles/metagenomics-compact-collision-2026-11-05.json`.
The next step is the complete internal native database/classification workflow,
not another replay of these accepted source witnesses.
