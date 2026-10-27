# Shared global thread controls

Owning local mainb7287f26d8d6f1d6e46bf580e850eb639d8a0ed0 remains private
and publish=false. This changes only the CLI worker-selection conduit,
its existing regression and README from green5. The already-published
common0.12.3 ThreadArgs owns global -t/--threads before/after subcommands.
The same requested value populates the same runtime input field.

Cargo.toml, Cargo.lock, sampling.rs and report.rs are byte-identical to
green5. Available-core default, pool/scheduling, numerical policy and engine
source do not change. All11 tests, strict Clippy, format and one release build
pass locally. Existing regression verifies all four alias/placement choices,
actual1/4 workers and identical report bytes. No new whole-BAM/oracle or
benchmark campaign was run for this CLI conduit.

Green5's four-native/source evidence remains bound to its original bytes.
Whole performance40592936910 measures that immutable binary, not the new
39dc3e339ef434370bf926c0f6dd9889601fb60ad5b91a92573af1c23bb118fb
Mac binary. Run the normal four-native gate here, not bootstrap; do not
rename or overwrite historical source/performance evidence. Registry adoption
and product publication remain blocked/pending their separate gates.
