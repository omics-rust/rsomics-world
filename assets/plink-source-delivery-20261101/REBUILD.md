# Private offline rebuild

This is an engineering delivery candidate, not a public release or legal
clearance. Original Rust is MIT OR Apache-2.0. The shim/pgenlib is LGPL-3.0-or-later;
SIMDe and every Cargo dependency retain their own terms and attributions.
See code/LICENSES.md, code/THIRD_PARTY_NOTICES.md, bundle.json and all dependency
LICENSE/NOTICE/AUTHORS files. licenses.json binds separately supplied original
noodles MIT notices and the complete CC0 text. Their authors are not replaced
by the application author.

Linux prerequisites: native Rust/Cargo1.91.0, a C++17 compiler and standard
development/system libraries, Python3.11+ and ordinary shell tools. Source,
build.rs, Cargo.lock, all85 dependency sources, build/proc-macro scripts and
complete existing test fixtures are included. No private clone, live registry
or upstream PLINK executable is needed.

Verify the delivered archive's published checksum before unpacking. From code/,
set CARGO_HOME, CARGO_TARGET_DIR and TMPDIR to fresh task-owned build directories
outside the delivered source, then run:

```sh
cargo test --release --locked --offline -- --include-ignored --nocapture --test-threads=1
```

The included .cargo/config.toml routes registry sources to ../dependencies.
Do not remove registry checksums or change Cargo.lock to make a build pass.
The one production executable is CARGO_TARGET_DIR/release/rsomics-plink.
Keep actual compiler/target/flags, complete build logs and resulting identities;
this is not a promise of bit-for-bit equality across toolchains or platforms.

Historical private-green-1 requires our development guard for ordinary local
macOS compilation. Private-green-2 includes the owning guard-policy correction:
downstream builds no longer need our SSD paths or guard marker, while explicit
guarded-OUT_DIR validation remains. bundle.json identifies the exact source
authority; do not substitute one revision's acceptance for another.

Our own Mac-mini operations always retain the external-disk guard. Linux
replacement success alone does not qualify Mac installation of this delivery;
final four-native source/binary binding and public installation/release checks
remain separate. No public binary/source pair has been published by preparing
this source bundle.
