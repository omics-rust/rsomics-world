# Locked raw-reader candidate: green-9

Uses the verified remote-generated gzip lock from run 36692452334, preserving
all old packages. Corrects the compressed-count cap fixture that previously
failed on the earlier position input; no production validation was relaxed.
Adds two documented literal-V1 contracts and actual synthetic grouped/no-ref
raw-to-stage-14 coverage alongside shipped subset/full coverage.

26 ordinary input contracts and three explicit external-oracle tests now join
the unchanged prepared-input core tests. Strict native TSV, deterministic
binary64 conversion and bytewise observation groups are the supported domain;
arbitrary R table/decimal/header heuristics remain excluded. No CLI, release,
ingestion throughput or complete inferCNV result is claimed.

Four-native debug/release, format, strict Clippy and actual pinned external
comparisons are pending. The original green-8 test failure, lock, review and
hash evidence are preserved in the controller ledger.
