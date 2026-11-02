# Private PLINK complete output and VCF UTF-8 candidate green-12

Complete committed source from clean private owner main
1906a48b8e6e70bf2478c8e3daf1a44c38c56572, tree
a090db0a33997d9d577b9eee6494f8ade44807df. All306 Git bodies and modes
are retained, including compact shared metadata storage.

Private ordered execution now transfers one complete result per lane and
returns that same payload after single-writer consumption. LD pair jobs are
byte-admitted with a4MiB result cap and serial fallback for an over-cap row;
make uses the same bounded ownership protocol. Prune retains its independent
stateful path. Numeric kernels, pair ordering and output formats are unchanged.

VCF raw record bytes are assembled across plain/BGZF read boundaries before
UTF-8 validation. Observed record width is propagated to reader workspace
admission; invalid UTF-8 and post-scan source growth remain explicit errors.
This does not claim complete memory admission for the metadata first pass.

Full local debug/release suites each pass13 library and3 integration tests,
zero failed or ignored, with zero main/doc tests. Formatting and strict
all-target/all-feature Clippy pass. The unchanged scientific workflow retains
six lanes,123 reports per worker profile and66 worker records.

No public API, dependency, version, C++ ABI, output schema, genotype/numeric
policy, generation hash or atomic/durable semantics change. The unresolved
default-atomic/--durable policy remains unresolved. Representative performance
is not inferred from these implementation or fixture checks; prior negative
performance evidence remains authoritative. Native-four and source-consumer
qualification remain separate gates for this exact source.

Archive members are sorted exact Git blobs/modes, regular USTAR only, with
uid/gid/mtime0 and empty owner/group names, gzipmtime0 and no directory,
scratch, AppleDouble or private-cohort entries. This capture does not dispatch
a workflow, accept performance, publish a product or waive any gate.
