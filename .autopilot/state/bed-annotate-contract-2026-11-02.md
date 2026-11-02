# BED annotate contract

No native implementation yet. Root reads retained exact BEDTools2.31.1
annotateBed.cpp, countListHits and extended-bin code, then independently
verifies all ten actual oracle exits and complete stdout/stderr identities.

Packet: `/Volumes/KIOXIA/Developments/tmp/bed-annotate-contract-20261102.dprmex`.
COMPLETE.json SHA256
`ba681bb738525ad8d930aa1a754ea2c7fdf1a2527325a345d85efedd7ac00722`.
Actual local executable is BEDTools2.31.1, SHA256
`fc994e1fcfdce8ce8591630d71f21fe67e2275c2a8d967dc0e3dfb4cfa4677a3`.
This is distinct from the retained remote source-built executable.

## Valid-input behavior

- Default fraction, counts and both modes emit separate results for each B
  file; both places count then fraction adjacent for each file.
- Fractions use f64 division and six decimal places, unlike coverage's f32/
  seven places. Repeated and nested B records retain count multiplicity.
- Output is byte-lexical chromosome, ascending source bin, then original
  within-bin A order, not coordinate or overall A-input order.
- The source bin shifts are14 then3, with offsets
  `[299503,37359,4681,585,73,9,1,0]`. These preserve the actual32678 literal
  in the first two source offset expressions; do not replace them with a
  different canonical-bin implementation from an old asset.
- A points use checked virtual `[p-1,p+1)` bounds and denominator2, printing
  original coordinates. B points participate in widened overlap counts but
  contribute zero covered bases. Point-only witness counts1/2/1 and fractions0.
- BED12 uses its outer span and retains all columns; no split option.
- Names enable a header. Both mode adds `_cnt`/`_pct`; nonempty A uses its
  complete field width, including BED12. Empty A still emits a names header,
  beginning `#\tfirst_cnt\tfirst_pct\tsecond_cnt\tsecond_pct\n` in both mode.
- Existing product coordinate bounds, header skipping and strict valid BED6
  strand behavior remain. Missing/unknown strand errors under selected filters
  are an existing documented product boundary, not inferred source parity.

The source fails to reject counts+both and mismatched names cardinality.
Ruling: retain the product's fail-fast valid CLI boundary rather than reproduce
broken-column output from invalid arguments. This does not require another
user choice. GFF/VCF/BAM, split, fraction thresholds, sorted execution and
source-header passthrough are excluded and unsupported flags must reject.

## Reuse boundary

Old annotate551632db1dce and map1fb4c6cc8d84 are fixture/reference assets, not
safe production cores. Discard permissive GFF guessing, dropped malformed rows,
unchecked/saturated coordinates, first-byte strand comparison, obsolete bins,
BED12 header truncation and unsafe map lifetime conversion. Preserve useful
oracle cases and provenance; do not revive their repositories.

Coverage and annotate share only the product-private positive clipped-span
depth/breadth primitive and existing candidate index. Point treatment, output
ordering, fractions and formatting stay local. Map's23 column reducers are a
separate later policy, not one generic annotation reducer. No new foundation,
public crate or dependency is justified.
