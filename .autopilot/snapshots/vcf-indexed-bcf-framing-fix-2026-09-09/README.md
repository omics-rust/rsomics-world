# Indexed BCF record-boundary repair candidate

Full four-native verification snapshot, not a product release. All 174 source
entries match the manifest. Only `src/regions.rs` differs from
`vcf-indexed-bcf-framing-red-2026-09-09`; every frozen CLI test is unchanged.

Expected-red run `34297832960`, exact world
`d449c86b369e2ae3928060f17c7d30c34fdfcd96`, fails only index selection on all
four native targets. Control CI `34297754328` passed. Linux ARM raw assertions
were read before production edits. Every target accepts all 16 malformed
commands and replaces their synthetic destination; eight valid controls pass.
Selection has 11 pass/one fail on Linux or nine pass/one fail on macOS. Other
focused concat 38, reheader 21, index 13, resource three and writer 11 groups
pass. This red deliberately skipped full debug/release, oracles and packaging.
All four artifacts and raw per-case evidence are independently verified.

Indexed concat now calls the existing private checked BCF reader over its
existing chunk reader, without another buffer. Indexed view constructs a
private checked BCF iterator with the pinned Noodles raw-candidate semantics:
validate the record's RID/name first, reject another reference, shortcut a
fully unbounded interval, otherwise inspect position and record end. Existing
typed decoding, counters, overlap/previous-region dedup and sparse empty-tail
skip remain unchanged. The VCF branch uses the identical inner query/records
mapping used by pinned noodles-util 0.82, retaining its existing boxing shape
without an additional wrapper. No new public item, dependency or foundation.

Two supplemental unit groups check sparse RID 2, record-span boundaries,
unbounded POS 0 and missing name errors before reference/interval shortcuts.
These are additional tests, not previously observed-red evidence. They have
not run yet. Independent source reviews, rustfmt and diff checks pass; runtime
and hot-path performance remain unverified for this candidate.

The CLI slice covers one contig/chunk/data frame, not disjoint chunks,
cross-frame malformed lengths, excluded corruption or ligation. It verifies
named-output preservation, not rollback of streaming stdout. Do not broaden
its fixed-scope claim beyond observed results.

| Artifact | SHA-256 |
|---|---|
| `source.tar.gz` (341102 bytes) | `eafd3c82497d748e567da2a89b310f276a2d2ecf40854caa3c870bfb81c763c4` |
| `files.sha256` | `063b240774ffc5dbbfc4ec0b8e27d859b56749bcddf2a4143a7f67f3d9d797af` |
| `tracked.patch` | `69538cb1768c0abaf3e801d2f05e84247a412d863fa860f189fd3e77d7613696` |
| `src/regions.rs` | `7188ed9ac337c5e4bd3d1b67517e09643e3d30062ffe726af38cf3291eeca2f3` |
| indexed CLI test | `12a550f651ba228edca366cf73b34d656106178f07bd6139173eed2c8de625b3` |
| concat CLI test | `06d7a94b237ab77eec2cceb6e18d7f8fd6a6e14510fb6cfd01d7f70a0187ef8c` |
| reheader CLI test | `a6cd1b893e7626385c03e290f0f99aa57f1aaee2bc8dd2747504799537768d18` |

Owning product HEAD is `682942cfa69768dc3a127a8544f2f07213b704ea`, index empty.
No local Rust/product execution: boot occupancy exceeds 95%, and all scratch
is external. Dispatch with `regressions_only=false`. Publication and formal
performance gates remain open.
