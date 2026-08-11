# `rsomics-vcf filter` implementation plan

## Contract

Implement the release 0.2 contract in `docs/10-products/variant.md` against
bcftools 1.24. Keep the expression engine and all filtering policy private to
`rsomics-vcf`. Do not expose `filter` in public help until the complete stable
surface is implemented and tested.

## Sequence

1. Import only useful historical fixtures and capture a focused live 1.24
   behavior matrix for expressions, annotation modes, genotype rewriting,
   masks, gap filters, and malformed input.
2. Build the private expression lexer, parser, typed AST, header binder, value
   algebra, and evaluator with ordinary unit tests for every operator and
   error boundary.
3. Integrate scalar, vector, sample, genotype, calculated-variable, aggregate,
   statistical, file-set, and regex evaluation against the existing typed
   VCF/BCF record model.
4. Implement streaming hard and soft filtering with transactional output and
   VCF/BGZF/BCF equivalence. Add filter-header generation and mode semantics.
5. Add failing-sample genotype replacement with checked `AC` and `AN` updates,
   then bounded SNP-gap and indel-cluster state.
6. Reuse the existing target and indexed-region machinery for targets,
   regions, and masks, preserving their distinct overlap defaults.
7. Add the unified `rsomics-help` command adapter, common JSON summary, public
   library options, README contract, and Linux x86_64 live 1.24 CI oracle.
8. Run formatting, strict Clippy, debug and release tests, rustdoc, clean
   packaging, representative VCF and BCF compatibility, repeated performance
   and RSS gates, four-native-target exact-head CI, registry publication, and
   fresh-install smoke verification.

## Commit boundaries

- fixtures and oracle matrix;
- expression syntax;
- typed expression evaluation;
- transactional filter stream;
- annotations and genotype rewriting;
- gap, mask, target, and region policy;
- CLI and compatibility matrix;
- performance evidence;
- release metadata and publication evidence.

Each commit must leave every public command correct. Private incomplete modules
may land with passing tests, but no unfinished filter flag or help entry may be
published.
