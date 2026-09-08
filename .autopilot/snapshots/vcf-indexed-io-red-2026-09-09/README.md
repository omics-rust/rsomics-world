# Indexed concat and writer error regressions

This snapshot changes two files relative to
`vcf-indexed-resources-bounded-red-2026-09-09`: the resource test harness and
unit tests inside `src/format/writer.rs`. Its production writer code is
unchanged. All other 170 source hashes match. A tentative local concat
preflight edit was withdrawn before capture; `src/concat.rs` matches the
previous snapshot byte for byte. No production ingestion changes exist yet.

Run `34283224823` completed on four native platforms, but is not accepted as
the intended scaling red. The checked Linux x86_64 log observed two threads
at one input, then failed the pipe-message assertion because the command
reported `invalid filters`, not `Broken pipe`. macOS ARM64 reported the same
wrong error. A local read-only multithreaded Python probe demonstrated that
macOS `ps -M` omits USER on continuation rows; the old parser missed those
threads. Native descriptor limits were 65536 on Linux x86_64 and 10240 on
macOS ARM64, sufficient for 256 readers. These are direct log observations,
not a completed performance or resource gate.

The corrected resource harness counts PID in either first or second column.
It now separates the three contracts: all four input-count observations and
bounded nonzero exit after pipe closure; explicit broken-pipe diagnosis; and
successful complete small merges. Diagnostic stderr is logged for every
resource sample. The existing 30-second first-record watchdog, ten-second
pipe-close deadline, five-minute workflow step limit and FD capacity check
remain. The test observer thread is outside the measured product process.

Noodles 0.90 wraps field-specific write errors inside `io::ErrorKind::InvalidInput`.
Its error sources retain the actual I/O cause, but our `map_write_error`
currently formats only the outer error and discards that cause. New unit tests
inject BrokenPipe, PermissionDenied and WriteZero at every byte of a valid
typed VCF record, requiring the original error kind and diagnostic with record
context. A zero-capacity BufWriter forces each write through to the test sink.
A separate malformed-filter test must remain an invalid-input error. Expected
failures are the resource one-thread assertion, the broken-pipe diagnostic,
and the injected-I/O classification; they have not yet been observed for this
snapshot. The already observed stdout regression remains included unchanged.

| Artifact | SHA-256 |
|---|---|
| `source.tar.gz` (324298 bytes) | `d6cb764cfc151891eb02afa5924c253b22f0d9de046c86af572c7c2e2fb5dad1` |
| `files.sha256` | `66376439f9241db62973894dfbde616811acc2e3fb00d69aae40dbb35e90e2d1` |
| `tracked.patch` | `f127cbf6fb8519f47368246070efce1ba774877bd5d01a6b8765d9fc3847303e` |
| `tests/concat_resources.rs` | `837fb616aad95913c5bc57de3a702b65375be470fc06e69742006cb8049311a3` |
| `src/format/writer.rs` | `9e51134e4e787f107cd0480588d2202695f93d228dd63e530028e8767f0ae97d` |

Run `regressions_only=true`; the four focused steps are independent and retain
separate logs. The 172-file source inventory is unchanged. All artifacts are
on external disks. Local Rust remains prohibited by boot-container occupancy.
The owning repository is unstaged on unpublished
`682942cfa69768dc3a127a8544f2f07213b704ea`. This is not a release, and all
publication and remaining concat gates still apply.
