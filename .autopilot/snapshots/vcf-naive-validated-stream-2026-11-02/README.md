# Private VCF validated-stream delta

These exact patches apply over the inherited 174-body naive-red candidate,
not over owning Git HEAD alone. The original before manifest is
`../vcf-naive-extra-red-2026-09-09/files.sha256`; its body hash is
`7d5649d0b15ff6f4e120d811eb90c9e8820f2926073e43bb633bffe54e38a9b8`.
After manifest SHA256 is
`4b6b695c7438be2c16028e12fe16d48d19c30f3d82e638557725e27603496ee1`.

Only `src/format/reader.rs`, `src/concat/naive.rs` and `tests/concat_cli.rs`
change. All 174 modes and 171 other bodies remain unchanged. These are
control-plane recovery assets, not an integrated owning release or Cargo
dependency. Do not apply them to a different base or overwrite inherited work.

Plan: `docs/plans/2026-11-02-vcf-naive-validated-stream.md`.
State: `.autopilot/state/vcf-naive-validated-stream-2026-11-02.md`.
One whole-feature review approves this bounded repair, not whole concat or
public delivery. Missing-newline policy, full compatibility/performance and
owning four-native exact-head gates remain open. No speed claim follows.
