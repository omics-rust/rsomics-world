# rsomics-table progress — 2026-08-20

Repository: `omics-rust/rsomics-table`

Completed product operations:

- `validate` at `519fc20`;
- `select` at `69c709c` plus deterministic stdout fix `550a3ad`;
- `filter` at `bba0295ab3f47cef2d7102a86d2e914a457c6bea`;
- `sort` at `584f7ffa25e6653e44637e97d9b34b3b24d0bfed`;
- `join` at `164e08b0630796c6658ce832e9f64b19bab5ab41`.

The filter, sort, and join heads passed debug and release tests on native Linux
and macOS for `x86_64` and `aarch64`. Exact-head CI runs were `32297305110`,
`32298162392`, and `32299619279`, respectively.

The join head packages from a separate external target directory as 42 files,
197.4 KiB uncompressed and 43.1 KiB compressed. The crate archive SHA-256 is
`43403c554da9c062c539f537e387075543ac983255464e585e307fe34e159a99`,
and `.cargo_vcs_info.json` names the exact join head.

Live csvtk 0.37.0 differentials cover select, filter, sort, and join. A
development sort calibration used the external 500,000-row fixture
`tables_500k.csv`, SHA-256
`1ba04c2706893f5ece09132298072dda2ee7c129cd7caef84aaf4646d848425a`.
Both implementations produced SHA-256
`7720ea84e11e2a8454d53a2a08bdc13014d06e8180b18a2a0aaf792c0b558a30`.
Five-run means were 285.0 ms versus 525.6 ms at one thread and 266.8 ms
versus 373.1 ms at four threads. This calibrates the implementation only; it
does not satisfy the final five-million-row, ten-pair release gate.

A development join calibration used that fixture as the left input and 6,000
unique rows as the right input, joining on all four fields. Both implementations
reproduced the input byte-for-byte. Five-run means were 215.7 ms for
`rsomics-table` and 286.4 ms for csvtk, a 1.33x throughput advantage. This is a
development calibration, not the final release benchmark.

No new public foundation was created. The strict table reader, field grammar,
expression engine, prepared sort keys, and deterministic sorter remain
product-private. `groupby`, the pinned-oracle CI job, attribution, and the final
performance/release gate remain incomplete.
