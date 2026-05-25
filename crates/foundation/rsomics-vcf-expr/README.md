# rsomics-vcf-expr

A `bcftools`-style VCF **filter-expression** parser and per-sample evaluator —
the shared expression engine behind `rsomics-vcf-setgt` (and other VCF tools that
accept `-i/-e/-t` expressions). Layer-A library only (no binary).

Parses expressions over INFO/FORMAT fields and standard columns
(e.g. `QUAL>20 && FMT/DP>=10`, `GT="het"`, `AF[0]<0.01`) into an AST, then
evaluates them per site and per sample.

## Use

```toml
[dependencies]
rsomics-vcf-expr = "0.1"
```

```rust
use rsomics_vcf_expr::{parse, Expr};
let expr = parse("QUAL>20 && FMT/DP>=10")?;
// evaluate `expr` against a parsed VCF record / sample column
```

## Origin

Independent Rust implementation of the bcftools filter-expression grammar, based
on the public bcftools `--include`/`--exclude` expression documentation and the
VCF spec, with black-box behaviour testing against `bcftools view -i/-e`. No
GPL/MIT upstream source was used as reference.

License: MIT OR Apache-2.0.
Upstream credit: [bcftools](https://www.htslib.org/) (expression syntax).
