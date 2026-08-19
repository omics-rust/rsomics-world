# rsomics-table progress — 2026-08-20

Repository: `omics-rust/rsomics-table`

Completed product operations:

- `validate` at `519fc20`;
- `select` at `69c709c` plus deterministic stdout fix `550a3ad`;
- `filter` at `bba0295ab3f47cef2d7102a86d2e914a457c6bea`;
- `sort` at `584f7ffa25e6653e44637e97d9b34b3b24d0bfed`;
- `join` at `164e08b0630796c6658ce832e9f64b19bab5ab41`;
- shared record-buffer reuse at
  `fb9becc6a430ce0008043de0f592d2f01d39c76d`;
- `groupby` at `90f57c4a9c71d9231f04d3f0d6bd35f929c95001`;
- unified table-output options at
  `acdc41154ea49912e34222a521a37d9c48ee1c62`;
- exact source attribution at
  `a2e7772d585cfd3379168a14775ca138b70e001e`;
- pinned source-oracle CI at
  `12bacee862c51678a6b37d1ea681b1532bdf9aa8`.

The filter, sort, and join heads passed debug and release tests on native Linux
and macOS for `x86_64` and `aarch64`. Exact-head CI runs were `32297305110`,
`32298162392`, and `32299619279`, respectively.

The record-buffer and groupby heads passed the same four native targets in
exact-head CI runs `32301836380` and `32302405199`. The local full suite also
passed live differentials against csvtk 0.37.0, GNU datamash 1.9, and bedtools
2.31.1.

The pinned-oracle head passed exact-head CI run `32305265384`. All four native
Linux and macOS `x86_64` and `aarch64` jobs built csvtk 0.37.0 from revision
`cc94b40d35cef9188d19f961718d9630479827c0` and passed debug, release, and live
csvtk differentials. The Linux oracle job additionally built GNU datamash 1.9
from its SHA-256-verified release archive and BEDTools 2.31.1 from revision
`705ccfdf2c9a77d71560c8adcece0663c2f5e18e`, then passed the complete ignored
compatibility suite. CLI integration tests cover the six exposed operations,
shared help sections, suggestions, non-ANSI output under `NO_COLOR`, and
machine-output separation. Source and license provenance is tracked in the
product's `THIRD_PARTY_LICENSES.md`.

The join head packages from a separate external target directory as 42 files,
197.4 KiB uncompressed and 43.1 KiB compressed. The crate archive SHA-256 is
`43403c554da9c062c539f537e387075543ac983255464e585e307fe34e159a99`,
and `.cargo_vcs_info.json` names the exact join head.

The groupby head packages from a separate external target directory as 49
files, 262.8 KiB uncompressed and 55.9 KiB compressed. The crate archive
SHA-256 is
`1a67c29d336179e1e1e67dbf040690a3094f142b9ed95fcd6e9631e28e93b15f`,
and `.cargo_vcs_info.json` names the exact groupby head.

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

A development groupby calibration used the same 500,000-row fixture. Global
grouping produced the same 200-row output as GNU datamash 1.9, SHA-256
`8ac1ced925fde58e51ae59a7ac5664c6b666056ea04f928a5424e94999cb8e4d`.
Five-run means were 68.2 ms for `rsomics-table` and 106.7 ms for datamash with
sorting, a 1.56x throughput advantage. On a key-sorted input with SHA-256
`c882a6677f810154668837e0a9eea0597d81fc8f61703a4a2c9fd3f3f89a71c5`,
consecutive grouping produced the same output in 36.9 ms versus 30.0 ms;
datamash remained 1.23x faster. These are development calibrations, not the
final high-cardinality and low-cardinality release gate.

No new public foundation was created. The strict table reader, field grammar,
expression engine, prepared sort keys, and deterministic sorter remain
product-private. The final representative performance gate, release review,
and publication remain incomplete.
