# Registry reset final gate

Status: verified on 2026-07-31 CST.

The registry reset is complete and the retained namespace matches the
product-family architecture.

## Live GitHub state

The `omics-rust` organization contains 17 `rsomics-*` repositories:

- six product repositories: `annotation`, `bed`, `fastq-preprocess`,
  `liftover`, `minimap2`, and `seq`;
- all nine retained public foundations;
- temporary `rsomics-igzip`;
- `rsomics-world`.

The organization metadata repository `.github` is outside the prefix count.
Twenty-three accepted product names intentionally have no repository yet.

## Live crates.io state

The registry contains 11 `rsomics-*` packages:

- all nine retained public foundations;
- the existing `rsomics-minimap2` product;
- temporary `rsomics-igzip`.

Every published version of those 11 packages is non-yanked. The other 28
accepted product names are absent from crates.io and remain planning
boundaries, not empty reservations.

## Post-reset boundary refinement

The source audit subsequently rejected the planning-only `rsomics-workflow`
boundary. Its sole historical candidate was a private sample-path TSV checker,
not a workflow engine or coherent utility family. The name was absent from
both GitHub and crates.io, so narrowing the allowlist from 30 to 29 products
changed no live repository, package, or user dependency.

## Source recovery state

The reset archives remain under:

```text
/Volumes/KIOXIA/Documents/omics-rust/_retired/registry-reset-2026-07-30/
```

The historical implementation pool remains available in the local clone
directory. New consolidation outputs are excluded by
`portfolio-consolidation-outputs.txt`, so regenerating the inventory does not
count reconstructed products as additional historical inputs.

## Verification

GitHub state was read from the organization repository API. crates.io state
was read from the public crate and version APIs, including every version's
`yanked` field. The local source count was regenerated independently on the
external disk: both the tracked and regenerated inventories contain 622 rows,
with identical crate-name sets.

No crate or repository was deleted, published, yanked, or created during this
gate.
