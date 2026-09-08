# Indexed concat resource regressions with bounded observation

This snapshot adds only `tests/concat_resources.rs` relative to
`vcf-indexed-stdout-red-2026-09-09`. All existing 171 source hashes are unchanged;
the inventory contains 172 files. No production changes have been made.

The suite runs indexed overlap concat with 1, 2, 64, and 256 inputs. A
131,072-record BGZF fixture keeps readers active while stdout applies
backpressure. After receiving the first record, it samples the child's
threads, numeric file descriptors, and resident memory. A test-process reader
thread bounds the first-record wait to 30 seconds, with child kill and reap on
timeout; that observer thread is not part of the measured child. Closing the
pipe must yield a nonzero broken-pipe error within ten seconds. Every sample
is printed before the one-caller-thread assertion. A Drop guard reaps the
child on assertion failure. The resource workflow step has a five-minute
timeout to contain remaining external probe and command-capture failures.

Linux uses `/proc` for threads/descriptors; macOS uses `ps -M` and numeric
`lsof -F f` entries. Both obtain RSS in KiB from `ps`. These are point-in-time
observations, not peak-memory or throughput benchmarks. The second test
verifies every record and coordinate of successful 32-record merges at all
four input counts. The workflow records and checks an open-file limit greater
than 259 before this suite. A capacity, timeout, or compilation failure must
not be interpreted as the expected thread-count assertion failure.

The one-thread assertion is expected to fail on the existing implementation;
it has not yet been observed remotely. Earlier stdout run `34282382683`
completed with all four jobs failing. Its Linux x86_64 job `102249965965` log
has been checked: 218 header bytes were emitted for plain VCF and corrupt
input 0; all other 30 concat CLI tests passed. A failing loop stops at its first
case, so this does not establish observed red coverage of all eight cases.

| Artifact | SHA-256 |
|---|---|
| `source.tar.gz` (323601 bytes) | `3c216c7f8c9bde43a7af4d0ebb3cedf5ffb379a2b9cccb479edb8ebd7078928e` |
| `files.sha256` | `3139c1836867160c5197e45e5107382b1344b6f24a319ac578e7f6925373b3ca` |
| `tracked.patch` | `b1086166df1242d58776bb02c70c0b58b782d4a20659077e755f45f85eb02950` |
| `tests/concat_resources.rs` | `392dc34f3fc8f998bda4c8255f90f4d0286f5870330a295a8eb73e3d1e62e8c5` |

The earlier `vcf-indexed-resources-red-2026-09-09` local draft lacked the
first-record watchdog, was never dispatched, and remains ignored. All source
and evidence are on external disks. No local Rust execution is permitted
while the Mac boot-container gate is closed. The owning VCF repository remains
unstaged on frozen unpublished commit
`682942cfa69768dc3a127a8544f2f07213b704ea`. Snapshots are audit inputs, not
releases; the byte-path publication hold and other concat gates remain open.
