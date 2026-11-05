# FASTQ quality-window candidate

## Current state

Owner: `/Volumes/KIOXIA/Documents/omics-rust/rsomics-fastq-preprocess`.
Main contains `a09e9af` (license originals), `e3a0d81`
(quality-window workflow), `ad92c90` (scope documentation), and
`a0796cae7a908cb3f6f18c323c9dc0c6d9421c6d` (CI diagnostics).
The latter two correct contradictory old exclusions/help and expose failed
release-test tails through check annotations. Owner worktree is clean.
No retained Cargo or quality benchmark process was running.

The previous execution completed front/tail/right combinations, SE/PE,
Phred33/64 and ordered one/four-worker oracle comparisons against pinned
fastp 1.3.6. Debug and release suites each passed 54 tests, alongside strict
Clippy, formatting and rustdoc. Whole-feature review identified post-poly
empty-read emission; the actual-oracle regression failed before correction
and passed afterward. No second review cycle is required merely to repeat it.

Representative million-pair measurements and raw samples are committed in
the owner's `benchmarks/2026-11-03-quality-windows.md` and corresponding JSON.
SE throughput improved; PE gzip was slower but used less memory. Do not
describe the latter as a throughput improvement.

## Delivery gate

On 2026-11-05, HTTPS push using `gh auth git-credential` failed with
`could not read Username`. `gh auth status` independently reported the active
Bengerthelorf credential invalid. No credential values were printed.
That was the initial observation, not the final state. SSH port22 closed;
the existing authenticated SSH443 route succeeded without changing credentials
or repository remotes. Both subsequent pushes succeeded through
`ssh://git@ssh.github.com/omics-rust/rsomics-fastq-preprocess.git`.
GitHub API authorization and crates.io authorization remain separate.

Run41962564002 at exact `ad92c9086beba679ee21031e07e86ae51fcc9243`
finished failed: Linux ARM, Linux x86 and macOS Intel passed; macOS ARM failed
the release test step. Job-log downloads returned403, including anonymous
access; the annotation only exposed exit101. Browser access was unavailable.
No cause or production fix is inferred from that evidence. The diagnostic-only
commit above did not change production logic. Its exact-head run41963314684
finished SUCCESS on all four native classes, including mandatory fastp tests
in debug/release and the declared Linux format/Clippy/docs/package/bench steps.
Do not rewrite the preceding failure as success or claim its cause was fixed.

## Fresh local package and installed workflow

Packet:
`/Volumes/KIOXIA/Developments/tmp/fastq-quality-install-20261105.3lXaUJvO`.
The clean `ad92c90` crate is54191 bytes, SHA-256
`db4ef23d56fbf7c2a96fe0d970c261167b263bf13e3bbc25a07340414eaf8475`.
All45 original Git bodies/modes match, plus Cargo's manifest original and VCS
metadata. The existing Table source verifier was reused; its body is pinned
in `package-verification.json`. `cargo install --locked --offline --path`
uses the independently extracted package and an external installation root.

Installed binary SHA-256
`b5bfc71c9f2e37b15ea3883554bd22e4412653552823e482bc2649e9b8f503f9`.
Fourteen native invocations pass, including one expected malformed-input
failure. Full output comparison covers SE1/4 workers, Phred64 normalization,
PE gzip, trim/filter composition and post-poly empty-read JSON accounting.
The first harness attempt wrongly assumed fastp prints version to stderr;
actual captured stdout proved otherwise. Its six calls remain retained;
corrected `lifecycle-attempt2` is separate and passed. No product fix was made.
`COMPLETE.json` SHA-256
`059ad11f47a3e90397662359c6fe42ee1827d5ccc9383d163f827c2798fd87d3`.

Fresh local strict Clippy/release54 tests and current debug54 tests pass with
required pinned oracle. The benchmark executable also passes smoke. Logs are
retained; no new representative benchmark was claimed for this installed binary.
The successful CI-only head and the older installed-package identity are not
conflated.

Run metadata pins in the packet:

- `ci-first.json`: `c5d8b7c260bf8f3f3859ecbe1de6fb8568bbaed28e6ccbd73b9475a647bb13f2`;
- `ci-final.json`: `3c518a76d7517b26bca02531c0e761b2af69537f78f97b53a86576dfbc808ec2`.

Version0.2.0 is not published. Native distribution/source/license delivery and
public installation remain future release gates; current registry0.1.1 is
not changed by these commits. Do not retry unchanged registry credentials.
