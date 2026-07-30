# FASTQ parallel-gzip product gate — 2026-07-30

Status: implementation and product documentation committed; exact-head
four-native-target CI green; not published.

## Scope

This gate addresses one measured product blocker in
`rsomics-fastq-preprocess`: compressed FASTQ output. It does not add a new
operation, crate, public foundation API, or advertised placeholder.

Historical `rsomics-fqgz` supplied an implementation asset. Its useful
chunked-libdeflate algorithm was classified as refactor-then-merge and
internalized in the product. The deleted micro-crate is not revived.

## Exact identities

- product implementation:
  `de07879d1d5ddaab9c5534e50d161ca660ba44e9`;
- product documentation head:
  `8e483fc9555627f0eee931063de9d94752a83520`;
- shared-contract alignment head:
  `f217fc4902b28b36f8d40eb96f894c459c9bcc43`;
- `rsomics-common`:
  `9f11f37c0fa48a24cae12549769f3395d9d0f19f`;
- `rsomics-help`:
  `c615aa8b85224055faad57c86abd068aded89d06`;
- `rsomics-seqio`:
  `b23cf8ad29fd06c84aaaa0c480ba1da8cff01e7d`;
- Rust: 1.91.0;
- fastp: 1.3.6, source
  `23d6211d4f05d61f561899f1b7702435a4b5d408`;
- libdeflater: 1.25.2;
- CI implementation run: `30551968781`;
- CI documentation-head run: `30552485149`;
- CI alignment-head run: `30569428189`.

Both CI runs passed native Ubuntu and macOS on `x86_64` and `aarch64`, with
formatting, strict Clippy, live fastp differentials, full tests, and benchmark
smoke. Linux `aarch64` has native correctness evidence from CI, but no
representative performance measurement.

## Machine and fixture

The representative performance host was
`dell-Precision-7920-Tower`, Ubuntu 22.04, Linux 6.8, `x86_64`, with two
Intel Xeon Gold 6238R CPUs. All source, toolchains, targets, temporary files,
fixtures, and results were under
`/data1/liangjy/rsomics-linux-x86_64-20260730`; the server root filesystem was
not used for build output.

Inputs:

- `SRR341550_1.fastq.gz`:
  `d7a15c1762d64a5434ced0cc665d7f5d167ca81a71e239f8237b9cd490dd7683`;
- `SRR341550_2.fastq.gz`:
  `18a8e61af21d276dfaf12035307d673e3f52c9f3ac57658ee2f593d1aabeb1a4`.

## Baseline diagnosis

The previous four-thread paired gzip path measured
`75.055 ± 1.086 s` over ten runs versus fastp's
`13.749 ± 0.411 s`. Peak RSS was 15.8 MiB versus 101.9 MiB, but rsomics used
only 113% aggregate CPU because `rsomics-seqio::Writer::gzip` compressed both
outputs serially.

The matched plain-output control measured `9.500 ± 0.233 s` versus fastp's
`11.275 ± 0.323 s`. This isolated compression as the blocker; the
transformation and strict-record pipeline was not rewritten.

## Selected design

The product retains `rsomics-seqio::Writer` for validation and FASTQ
serialization. A private `ParallelGzipWriter`:

- buffers 256 KiB chunks;
- compresses at most 16 pending chunks through the command's local Rayon pool;
- collects indexed parallel results in source order;
- emits standards-compliant concatenated gzip members;
- produces a valid gzip stream for zero surviving reads;
- propagates compression, downstream write, flush, and finish errors;
- remains inside the transactional no-clobber output path.

This uses the command's local product pool instead of adding hidden background
workers or mutating Rayon's process-global pool. A public `rsomics-seqio` item
was rejected because there is only one current consumer of the
thread-controlled contract.

## Correctness and interoperability

Paired decompressed outputs matched the aligned fastp slice:

- R1:
  `f13cb655feedf78cf1f3c512675ad73323409f5862b0b3a6e5e3d48e21e6e365`;
- R2:
  `452c78a98878e56bf1e5e7728b749e0277e0e14607fa465f7da3e83e551c078c`.

Single-end output matched:

- R1:
  `9cc5172922740e7291bdf9fdfadc3d03370665fb0a8d4d4c4c5d4b930c800b58`.

The files passed `gzip -t`. SeqKit 2.13.0 read the R1 output as 6,162,791
FASTQ DNA records totalling 622,441,891 bases, and fastp 1.3.6 consumed the
same concatenated-member stream successfully.

Candidate paired file sizes were 92,968,694 and 93,171,622 bytes. They were
0.07% and 0.06% above fastp's corresponding files, and approximately 1.3%
above the previous serial zlib-rs output.

## Performance result

Times are Hyperfine means and sample standard deviations after warmup. Peak
RSS is a separate `/usr/bin/time -v` run.

| Mode | Threads | Runs | rsomics | fastp 1.3.6 | RSS, rsomics / fastp |
|---|---:|---:|---:|---:|---:|
| paired | 1 | 5 | 22.308 ± 0.610 s | 39.091 ± 0.894 s | 31.5 / 88.7 MiB |
| paired | 4 | 10 | 10.863 ± 0.298 s | 13.891 ± 0.447 s | 31.5 / 101.9 MiB |
| single | 1 | 5 | 9.910 ± 0.186 s | 6.849 ± 0.090 s | not recorded |
| single | 4 | 5 | 5.360 ± 0.075 s | 4.937 ± 0.721 s | 19.6 / 52.9 MiB |

Decision:

- paired compressed output passes both throughput and memory gates;
- single-end compressed output does not claim a throughput win on this host;
- single-end passes a material resource-use gate through 63% lower measured
  peak RSS at four threads;
- the implementation is retained, but publication remains blocked on the
  unpublished foundation revisions, end-to-end QC handoff, final API review,
  and release-level performance decision.

## Raw evidence

Remote result directory:
`/data1/liangjy/rsomics-linux-x86_64-20260730/results`.

Tracked checksums of the principal Hyperfine JSON files:

- serial paired baseline:
  `f1857ccb33870ac6c435af98132fabc679266dcc97ebda9e676b3ccb8642b0d8`;
- paired plain control:
  `66d44add94fc3a2a9778cb9d06991945c1deef0890cf73ea518551c34a1c33d6`;
- paired four-thread candidate:
  `9b2ad1f59eaca8e37e673b2de45ab2e30638257bd05e1ee4341b304cee7850aa`;
- paired one-thread candidate:
  `e34c94e08a812e87b65959c478127d57d00b33c41cb4d70099e3debca00decb9`;
- single four-thread candidate:
  `72a4655de0936a11d01599c63b196f9e7e6a0e90d21c94d2e0804e5757a046be`;
- single one-thread candidate:
  `9107b7361765c76f33d07a00afcd7eb983e396b8cbc508281f52e16aa0bd6d30`.
