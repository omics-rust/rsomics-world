# Empty-tail BCF/CSI regression candidate

Focused four-native diagnostic snapshot, not a product release. Only
`tests/index.rs` differs from `vcf-sparse-index-fix-2026-09-09`; all 172 archive
entries and hashes were verified. The preceding repair is independently
running full diagnostics as `34290416281`, world
`47de686b0d4606992010cf07887745129804ee3d`. Its completion is not assumed.

Three added integration groups cover externally shaped CSI with six slots
for raw record IDs 2/2/5 and zero slots for a completely empty file. The BCF
header declares contig IDs 5/2/9. Index creation uses independent Noodles
indexing rather than the product's builder. Header and raw IDs are checked.

- `view` and overlap `concat`, each with v/z/b/u output, query either the
  declared empty tail or a mixed populated/empty selection. All 32 cases are
  attempted when command execution fails; successful outputs are decoded
  independently with Noodles and their JSON counters checked.
- `index --stats --all` includes declared contigs beyond the index span in
  raw-ID order without unnamed holes. Default statistics and total counts
  remain covered. This is a product contract; the pinned upstream `--all`
  crash is not emulated.
- Unknown regions and corrupt CSI still fail in both operations and all
  four encodings without replacing an existing destination (16 cases).

The preceding pinned bcftools/HTSlib 1.24 diagnostic already observed empty
RID-9 query success versus rsomics exit 4. These new test groups have not yet
been executed. Production query/statistics code remains unchanged from the
preceding repair. No extra public API or dependency was added.

Primary references: [CSI layout](https://github.com/samtools/hts-specs/blob/da617203a9527537746e200abda2885bec3a822c/CSIv1.tex)
and [HTSlib 1.24 indexed iteration](https://github.com/samtools/htslib/blob/1.24/hts.c#L3217-L3254).
The latter marks an iterator finished for a valid resolved ID outside the
index span. The observed HTSlib CSI shape is not an index corruption.

| Artifact | SHA-256 |
|---|---|
| `source.tar.gz` (330942 bytes) | `0075e6d267f9cc6dcf001d86689529f7bf37605b3587e56b9e184710ad074a43` |
| `files.sha256` | `4335ef27ab8a36738f02f23ce0cd756be8db8b01db4c9c86983472497b3bd87b` |
| `tracked.patch` | `669373ea4be527c3f6c23d18bac017e1bef78f4807cb2a5cb865da41f1076021` |
| `tests/index.rs` | `f4bed365b932d3a3ad2ac2370a522bc05ce616cbdcfa2bd500a8b6c3f2680dc9` |

Owning product HEAD/index are unchanged at
`682942cfa69768dc3a127a8544f2f07213b704ea`. Run this red candidate with
`regressions_only=true`; full debug/release/oracle/package gates will run
after the independently observed red is fixed. Local rustfmt and diff checks
pass; no local Rust build/test or product binary ran. Boot APFS occupancy is
over 95%, so compilation remains on GitHub's four native runners.
