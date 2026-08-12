# Table product dossier

Status: source and upstream-operation audit complete. No target repository has
been created.

## Boundary

`rsomics-table` is one CSV/TSV product for selecting, filtering, joining,
ordering, aggregating, reshaping, validating, and rendering delimited tables.
Its operations share one record model, field-selection grammar, delimiter and
header policy, transactional output layer, and installation identity.

The primary behavior sources are:

- [csvtk 0.37.0](https://bioinf.shenwei.me/csvtk/usage/) for header-aware
  CSV/TSV operations, field expressions, rendering, and Go `encoding/csv`
  compatibility;
- [GNU datamash 1.9](https://www.gnu.org/software/datamash/manual/datamash.html)
  for grouped aggregation, crosstab, transpose, reverse, and structural
  validation;
- [bedtools 2.31.1 groupby](https://bedtools.readthedocs.io/en/latest/content/tools/groupby.html)
  and [the BEDTools suite](https://bedtools.readthedocs.io/en/latest/content/bedtools-suite.html)
  for contiguous group aggregation and comma-list expansion.

`rsomics-table` does not become a generic dataframe engine. Stateful
expression evaluation, spreadsheet editing, plotting, and interactive
monitoring are not prerequisites for its first release.

## Upstream operation map

Multiple upstream spellings that express one user operation collapse to one
subcommand.

| Target operation | Upstream operations | Decision |
|---|---|---|
| `inspect` | csvtk `dim`, `nrow`, `ncol`, `headers`; parts of `summary` | one structural and summary report |
| `concat` | csvtk `cat`, `concat` | one row-concatenation command with checked schemas |
| `select` | csvtk `cut`; datamash per-line `cut` | one field-selection grammar |
| `filter` | csvtk `filter`, `filter2` | one typed expression command |
| `grep` | csvtk `grep` | exact, regex, pattern-file, and inversion modes |
| `head` | csvtk `head` | streaming prefix selection |
| `intersect` | csvtk `inter` | table-row set intersection, unrelated to genomic intervals |
| `join` | csvtk `join` | inner, left, and full joins with duplicate-key semantics |
| `sample` | csvtk `sample` | deterministic seed and explicit fraction or count |
| `split` | csvtk `split` | split by field value with transactional file creation |
| `uniq` | csvtk `uniq`; datamash `rmdup` | stable first-N rows per selected key |
| `freq` | csvtk `freq` | frequency table over selected fields |
| `sort` | csvtk `sort` | lexical, numeric, natural, reverse, and multi-key ordering |
| `shuffle` | csvtk `shuf` | deterministic seeded shuffle |
| `validate` | datamash `check`; strict csvtk reader behavior | width, quoting, delimiter, header, and encoding checks |
| `repair` | csvtk `fix`, `fix-quotes`, `del-quotes` | explicit opt-in repair; never a parser default |
| `convert` | csvtk `comma`, `csv2tab`, `tab2csv`, `space2tab` | delimiter and record-format conversion |
| `mutate` | csvtk `comb`, `fmtdate`, `mutate`, `mutate2`, `mutate3`, `rename`, `rename2`, `replace`, `round` | typed column transformations under one expression model |
| `reshape` | csvtk `fold`, `gather`, `sep`, `spread`, `unfold` | long/wide and field/list reshaping |
| `transpose` | csvtk and datamash `transpose` | one canonical matrix transpose |
| `reverse` | datamash `reverse` | reverse fields per record |
| `expand` | bedtools `expand` | zip-expand list-valued fields |
| `groupby` | csvtk `summary`; datamash and bedtools `groupby` | one grouped aggregation engine |
| `crosstab` | datamash `crosstab` | long-to-wide pivot with optional aggregation |
| `correlate` | csvtk `corr`; datamash covariance and correlation operations | table-oriented pairwise statistics |
| `to-json` | csvtk `csv2json` | JSON records or objects keyed by a field |
| `render` | csvtk `csv2md`, `csv2rst`, `pretty` | `markdown`, `rst`, and terminal presentation formats |
| `xlsx` | csvtk `csv2xlsx`, `xlsx2csv`, `splitxlsx` | deferred optional spreadsheet feature |

csvtk `plot`, its plot variants, and `watch` are excluded: visualization and
interactive monitoring are separate interfaces, and their inclusion would
pull terminal and graphics policy into the table core. `genautocomplete` and
`version` are CLI infrastructure rather than product operations.

Datamash per-field base64, digest, pathname, binning, and elementary numeric
functions are not separate subcommands. They may become functions of
`mutate` when a concrete bioinformatics workflow needs them.

The canonical `groupby` contract initially retains the historical
implementations of `sum`, `min`, `max`, `absmin`, `absmax`, `range`, `count`,
`first`, `last`, `unique`, `collapse`, `countunique`, `mean`, `geomean`,
`harmmean`, `mode`, `antimode`, `median`, `q1`, `q3`, `iqr`, percentile,
population and sample variance or standard deviation, median absolute
deviation, skewness, and kurtosis. Datamash operations `rand`, `ms`, `rms`,
`jarque`, `dpo`, `scov`, `pcov`, `spearson`, `ppearson`, and `dotprod` remain
undocumented until implemented and checked against the pinned oracle.
Historical bedtools spellings `distinct_sort_num`, `distinct_sort_num_desc`,
`concat`, `freqasc`, and `freqdesc` remain compatibility aliases or operation
options only after their current goldens are reconfirmed; they do not become
separate subcommands.

Equal keys are aggregated globally by default and emitted in deterministic
byte-sorted key order, matching csvtk `summary`. An explicit consecutive mode
provides the pre-grouped run semantics used by datamash and bedtools without
making that easily missed precondition the product default. Sorting is also
explicit: the engine does not silently reorder input to make consecutive mode
look global. Crosstab cells use global aggregation by default; the datamash
first-run behavior for a cell that reappears later is compatibility behavior,
not the canonical data model.

## Format and execution model

- CSV and TSV share one byte-preserving record stream.
- Delimiters are explicit single bytes; TSV is not inferred from an arbitrary
  filename.
- Header and no-header modes use the same signed index, range, name,
  exclusion, and fuzzy-name grammar across operations.
- Strict mode rejects malformed quoting, ragged rows, invalid field
  references, and output aliases. Repair and row skipping require explicit
  options.
- CRLF normalization, quoting, comments, empty records, duplicate headers,
  embedded newlines, and trailing fields have golden coverage.
- Text interpretation is operation-specific. Byte-preserving transforms do
  not require UTF-8; regex, JSON, display, date, and expression operations
  validate the text they consume.
- Named outputs are transactional. Multi-output operations stage the complete
  set before committing any path.
- The first release supports plain and gzip streams. Other codecs remain
  product-private and require compatibility and performance evidence.
- `rsomics-help` owns CLI presentation. `rsomics-common` owns errors, exit
  mapping, and output transactions. Data conversion to JSON is the `to-json`
  operation; an execution-result JSON envelope may use stdout only when table
  data is written to a named path.

`rsomics-csvio` is not retained as a public foundation. After product collapse
it has one consumer, so its Go-compatible reader, writer, field grammar,
newline handling, and display-width code move into private `io`, `fields`, and
`render` modules. No new public table-I/O crate is created.

## Target structure

```text
src/
├── cli.rs
├── io/
│   ├── input.rs
│   ├── output.rs
│   ├── record.rs
│   └── dialect.rs
├── fields.rs
├── expression.rs
├── aggregate.rs
├── operations/
│   ├── select.rs
│   ├── filter.rs
│   ├── sort.rs
│   ├── join.rs
│   ├── groupby.rs
│   └── validate.rs
└── render/
```

Operation modules remain private until another product demonstrates a
policy-free API. `rsomics-bed` may be composed with `rsomics-table` through
pipes and files, but neither product depends on the other.

## Historical asset dispositions

All 16 routed repositories were clean at the recorded inventory revisions.
The extra `rsomics-csvio` source was also clean at
`0fccfb8cc2085a117ae88dc4b993c8b71b9c693b`.

| Historical asset | Revision | Disposition |
|---|---|---|
| `rsomics-bed-expand` | `23aa4ee69ab6` | refactor algorithm and goldens into `expand`; discard duplicate CLI/help shell |
| `rsomics-bed-groupby` | `30cf021d1c59` | refactor aggregators into canonical `groupby`; preserve bedtools cases |
| `rsomics-tsv-crosstab` | `da8e867f60fc` | refactor then merge using the shared aggregation engine |
| `rsomics-tsv-filter` | `f694c99adab0` | refactor then merge; replace product-specific reader and flags |
| `rsomics-tsv-freq` | `1fdf44c8c55e` | refactor then merge with shared field keys and writer |
| `rsomics-tsv-grep` | `7bd579b36a6e` | refactor then merge; retain csvtk pattern goldens |
| `rsomics-tsv-join` | `635603c8e2ff` | tests and benchmark asset only; rewrite duplicate-key, join-type, CSV, and malformed-row behavior |
| `rsomics-tsv-json` | `89df50e0cb58` | refactor then merge as `to-json` |
| `rsomics-tsv-md` | `e9883846a90a` | refactor renderer and goldens into the shared private render core |
| `rsomics-tsv-pretty` | `c2068546e090` | refactor renderer and goldens into the shared private render core |
| `rsomics-tsv-rst` | `327cc7c8b055` | refactor renderer and goldens into the shared private render core |
| `rsomics-tsv-select` | `ba997aa55e05` | tests and benchmark asset only; rewrite around the shared field grammar |
| `rsomics-tsv-sort` | `1df47552324b` | refactor then merge natural and multi-key ordering |
| `rsomics-tsv-stats` | `108d43936350` | refactor aggregators into `groupby`; remove whole-input policy where streaming suffices |
| `rsomics-tsv-transpose` | `c46be5779685` | refactor then merge the span-based transpose and its goldens |
| `rsomics-tsv-uniq` | `85bec590cf28` | refactor then merge with shared key and row semantics |
| `rsomics-csvio` | `0fccfb8cc208` | internalize; preserve compatibility tests; remove public crate boundary |

No retired micro-crate repository is revived. Useful Git history is recorded
in the merge commit and the source table above.

### Live source audit

The revisions above were inspected at source level rather than accepted from
their READMEs or historical benchmark prose.

- `rsomics-csvio` contains useful Go-style writer rules, field-expression
  parsing, strict-quote fixtures, CRLF cases, and display-width tables. It is
  not a streaming abstraction: its consumers first load and normalize the
  complete input, and lenient parsing is explicitly only a best effort through
  the Rust `csv` parser. Those compatibility assets move into the product, but
  the reader state machine and ownership model are redesigned around a bounded
  record stream.
- `tsv-select` and `tsv-join` are TSV line splitters, not implementations of
  the target contracts. Select supports only positive indices or exact header
  names and silently fills absent fields. Join implements only an inner join,
  overwrites duplicate right keys, accepts ragged rows, and can emit a blank
  value for an absent left key. Their small fixtures and benchmark recipes are
  retained; their operation code is discarded.
- `tsv-filter` implements csvtk's older single-field numeric condition syntax,
  not the target typed expression language. It, `grep`, `freq`, and `uniq` all
  load the full input and stage the full output even when the operation itself
  is streamable. Their field, matching, ordering, and four-platform csvtk
  goldens are useful, but execution is rebuilt on the shared record stream.
- `tsv-sort` has strong csvtk fixtures for lexical, numeric, natural,
  multi-key, Unicode, quoting, and malformed-input behavior. It deliberately
  ports csvtk's unstable quicksort, including its deterministic permutation of
  equal keys. That comparator and evidence are retained. The target must pin
  this upstream tie order rather than claim stable input-order ties; date and
  user-defined order types remain absent from the historical code.
- `bed-groupby` streams consecutive groups but copies every selected value into
  a `String`; `tsv-stats` reads the complete file, stores every numeric value,
  treats a missing field as an empty value, and buffers complete output. Their
  overlapping reducers are not merged side by side. One typed accumulator
  engine keeps constant-space states for composable statistics and bounded
  per-group values only for exact quantiles, collapse, and other inherently
  materializing operations. Bedtools numeric formatting and datamash
  higher-moment tolerances remain separate compatibility policies.
- `tsv-crosstab` interns byte keys and has useful datamash cases, but it reads
  the full input, keeps numeric and textual cell values, and in consecutive
  mode silently retains only the first run when a cell key reappears. Its pivot
  layout and fixtures are retained; it must consume the canonical accumulator
  and grouping modes rather than keep a third reducer implementation.
- `tsv-transpose` has a useful byte-span and blocked-output algorithm, but it is
  TSV-only and necessarily materializes the matrix. The three renderers have
  substantially stronger four-native-platform csvtk differential tests and
  measured gates, yet duplicate record measurement, display-width, and output
  plumbing. They become adapters over one private rendering core rather than
  three copied implementations.
- `bed-expand`, `to-json`, and the remaining csvtk-derived commands contribute
  targeted algorithms and unusually useful edge-case goldens. Upstream quirks
  such as malformed JSON from unescaped key bytes are recorded as compatibility
  modes, not silently adopted as the safe default.

The old command shells use the removed `rsomics-help::HelpSpec` API, and eleven
repositories contain the same `main.rs` byte for byte. All CLI/help shells are
discarded in favor of one `rsomics-help`-driven product CLI. Narrating comments,
audit history, and comments that merely restate control flow are not migrated.

Only the csvtk-derived filter, freq, grep, JSON, Markdown, pretty, rST, sort,
and uniq repositories currently install a pinned oracle across all four native
CI target classes. The bedtools/datamash assets and the select/join micro-crates
run Ubuntu-only CI and skip their live differential tests when the oracle is
absent. Consequently, no historical green badge substitutes for the target
repository's release gate.

## Retained evidence

Historical measurements are migration assets, not performance claims for the
future consolidated binary.

| Operation asset | Strongest retained evidence | Use in target |
|---|---|---|
| `bed-groupby` | 1.82 times faster than bedtools 2.31.1 on 1M BED5 rows | remeasure after aggregation merge |
| `tsv-crosstab` | byte-identical count and sum; 1.81 and 1.46 times faster than datamash 1.9 on 5M rows | retain fixture and live oracle |
| `tsv-stats` | about 1.97 times faster than datamash on a 5M-row grouped workload | retain fixture; replace materializing policy |
| `tsv-transpose` | byte-identical; 3.42 times faster and about 45% lower RSS on a 562 MB matrix | retain algorithm, fixture recipe, and oracle |
| `tsv-md` | byte-identical; 7.43 times faster and much lower RSS on Apple M2 | retain complete gate |
| `tsv-rst` | byte-identical; 3.69 times faster and much lower RSS on Apple M2 | retain complete gate |
| `tsv-pretty` | byte-identical; 1.73 times faster but higher RSS on Apple M2 | retain throughput and memory tradeoff |
| `tsv-select`, `tsv-join` | historical 1.41 and 1.50 times throughput results from dirty revisions | benchmark recipe only |
| `bed-expand` | historical 2.44 result with unresolved oracle-version provenance | fixture and command recipe only |
| remaining assets | live-oracle tests and benchmark harnesses without a release-grade recorded decision | preserve and supersede |

Consolidation changes dispatch, parsing, buffering, and dependency shape, so
every stable subcommand receives a fresh target-head measurement.

## First release slice

The first implementation slice is:

- `select`;
- `filter`;
- `sort`;
- `join`;
- `groupby`;
- `validate`.

This slice exercises the shared reader and field grammar, streaming and
materializing operations, multi-input semantics, aggregation, strict failure,
transactional output, and the unified CLI layer. `join` must cover inner,
left, and full joins plus duplicate keys before it is documented.

## Compatibility and performance gates

- Build pinned csvtk 0.37.0, datamash 1.9, and bedtools 2.31.1 where their
  operations are relevant.
- Retain frozen goldens, but run the real oracle in CI rather than treating
  optional local binaries as sufficient.
- Differential fixtures cover CSV and TSV, quoted delimiters, embedded
  newlines, CRLF, comments, headers, no headers, duplicate names, duplicate
  join keys, empty input, ragged rows, invalid UTF-8 where applicable, and
  transactional failure.
- Streaming operations use at least five million non-trivial records.
- Sort and join gates include repeated keys, csvtk's pinned deterministic
  unstable tie permutation, numeric and natural ordering, and outputs large
  enough to exercise real buffering.
- Grouped operations measure sorted streaming separately from explicit
  sorting; no unused `--threads` option is exposed.
- Each result records exact revisions, machine, input generator and hashes,
  flags, output equality, timing distribution, CPU time, peak RSS, and
  compression mode.

GPL-covered datamash source is not copied. Its public documentation and
black-box executable are behavior references. csvtk and bedtools are MIT;
their names, versions, and relevant notices remain in product attribution.

## Explicit exclusions

- No operation-sized public crates or public `rsomics-csvio`.
- No implicit repair or silent malformed-row skipping.
- No plotting, interactive watch mode, or terminal dashboard.
- No dataframe query language or in-memory dataframe public API.
- No direct dependency on another Layer B product.
- No advertised operation before its target implementation, live oracle, and
  representative performance gate pass.
