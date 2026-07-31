# rsomics-world

`rsomics-world` is the control plane for `omics-rust`: a set of coherent,
high-performance Rust bioinformatics products and their shared foundations.

The project previously published hundreds of operation-sized crates. That
partition was rejected because it confused a Rust module boundary with a user
product, repository, CI pipeline, version, and installation boundary.

The current rule is:

> One recognizable product or workflow family per Layer B crate. Operations are
> subcommands and modules. A public Layer A foundation exists only when at least
> two products consume it.

## Current portfolio

- 30 planned product-family boundaries.
- Nine long-term public foundations.
- `rsomics-igzip` temporarily retained because a published `rsomics-seqio`
  version depends on it.
- Historical operation implementations preserved on external storage as a
  source pool for consolidation.

The live and planned namespace is indexed in [REGISTRY.md](REGISTRY.md).

## Repository model

Each public product or foundation is an independent repository under
[omics-rust](https://github.com/omics-rust). There is no Cargo monorepo,
submodule aggregation, or cross-repository path dependency.

Local clones live under:

```text
/Volumes/KIOXIA/Documents/omics-rust/
```

`rsomics-world` contains documentation, product dossiers, audit scripts, and
durable campaign state. It contains no product implementation.

## Reconstruction

Existing implementations are reused rather than discarded. The generated
[portfolio inventory](docs/00-overview/portfolio-inventory.tsv) maps the local
source pool to provisional target products. The
[portfolio reconstruction](docs/00-overview/portfolio-reconstruction.md)
explains the 30-family model and shared-foundation evidence.
The [product dossier index](docs/10-products/README.md) tracks the mapping from
historical implementations to target products, and the
[consumer-driven foundation audit](docs/01-foundations/consumer-driven-audit.md)
records the current public API risks and migration waves.

For each product:

1. survey real upstream operations;
2. deduplicate overlapping tools;
3. map old implementations into target modules and subcommands;
4. preserve compatibility tests, fixtures, and benchmarks;
5. evolve shared foundations through concrete consumers;
6. publish only after the product interface and evidence are complete.

## Start here

- [AGENTS.md](AGENTS.md) — unattended operating rules
- [CONVENTIONS.md](CONVENTIONS.md) — architecture and engineering conventions
- [ROADMAP.md](ROADMAP.md) — reconstruction phases
- [TODO.md](TODO.md) — current executable checklist
- [REGISTRY.md](REGISTRY.md) — live and planned public namespace
- [docs/](docs/) — upstream surveys and product dossiers
