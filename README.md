# rsomics-world

Mission control for **rsomics** — a campaign to reimplement the C/Python/R
bioinformatics toolchain in Rust as `rsomics-<name>` single-binary CLIs, each
faster than the upstream it replaces.

This repo is **not** the code. It is the control plane: planning, conventions,
the operating manual, perf provenance, and the [crate registry](REGISTRY.md).
Every tool and library lives in its **own** repo under
[omics-rust](https://github.com/omics-rust) and publishes independently to
crates.io. There is no monorepo, no workspace, no submodule aggregation — flat,
independent repos, so a crate's code only ever lives in that crate's repo.

## Layout

| What | Where |
|---|---|
| This control plane | `omics-rust/rsomics-world` — `CLAUDE.md`, `CONVENTIONS.md`, `ROADMAP.md`, `docs/`, `scripts/`, `REGISTRY.md` |
| Each crate | `omics-rust/rsomics-<name>` — own repo, own CI, own crates.io release |
| Foundation libs | `rsomics-common`, `rsomics-bamio`, `rsomics-intervals`, `rsomics-pileup`, … (consumed via crates.io) |

## Install a tool

```sh
cargo install rsomics-<name>
```

## The contract

Every crate that ports an upstream tool ships with:

- `tests/compat.rs` — byte-or-field-exact diff against the upstream binary.
- a perfgate record proving **strictly `> 1.0×`** throughput vs that upstream on
  the same machine, same input, same flags. Equal-to-upstream is a failure.
- `## Origin` documenting clean-room methodology for GPL upstreams.

See [`CONVENTIONS.md`](CONVENTIONS.md) and the per-domain plans in [`docs/`](docs/).

## License

Each crate is dual-licensed [MIT](LICENSE-MIT) OR
[Apache-2.0](LICENSE-APACHE-2.0).
