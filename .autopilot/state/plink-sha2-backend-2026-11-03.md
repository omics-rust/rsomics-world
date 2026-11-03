# PLINK maintained SHA-256 dependency candidate

Implementation admission only; no new candidate, throughput/RSS improvement,
native/source qualification or publication is claimed here.
Plan: `docs/plans/2026-11-03-plink-sha2-backend.md`.

## Baseline and reason

Private base270caf52d2aa87375cfd1ab5c6ef8d0ca2acf29c is green15 native/source
qualified. Its complete paired comparison has no material cohort RSS advantage
over7cee; do not pursue further capacity-only changes as an RSS fix. The older
e210 one-off attribution identifies output hashing as a substantial coarse
interval, not a current precise crypto-only cost. Preserve SHA-256 and every
durability guarantee rather than deleting useful work to meet a timing target.

Root reads cached sha2-0.11.1 manifest/changelog/default runtime dispatch and
software-unroll source, and verifies the official tagged
[manifest](https://raw.githubusercontent.com/RustCrypto/hashes/sha2-v0.11.1/sha2/Cargo.toml)
and [dispatch](https://raw.githubusercontent.com/RustCrypto/hashes/sha2-v0.11.1/sha2/src/sha256.rs).
MSRV1.88 fits product Rust1.91. Default runtime CPU checks/hardware/software
backends differ from0.10.9; no CPU feature is forced and no speedup is inferred.
SHA-512's AVX2 backend is not an x86 SHA-256 acceleration claim.

Fresh public sparse read under the existing external guard confirms exact
sha2-0.11.1 is not yanked, rust_version1.88, checksum
47d7069beb7d6ac7b9acd1039986e73443f24234f41074da099d6f994ac9ad19.
Only metadata is read; no dependency archive, credentials or shared config is
downloaded/copied/changed. Cached sources are already external.

## Bounded candidate scope

Pin sha2 exactly0.11.1 with unused default features disabled; update its lock
closure offline. Only Cargo.toml/Cargo.lock may change, plus transaction.rs
solely if necessary for the same incremental Digest/lowercase output API.
No custom crypto, archived sha2-asm, unsafe product code, schema/algorithm/
64KiB-read/worker/codec/atomic-I/O changes or new public component.

Accepted green15 consumers are the unchanged baseline; do not rerun them merely
to manufacture a RED. Desired behavior is identical, so do not add tests of
manifest dependency text or a scalar helper matrix. Final existing golden,
tamper, transaction and full CLI consumers must pass with the actual candidate.
Run external guarded fmt/strict Clippy/full debug+release, no network retry,
compiler fallback, SSH, performance campaign, stage/commit/publish by producer.
Initial missing-cache/resolution failures stop that affected action for bounded
inventory, not an automatic online fetch. Report unrelated lock movement.

Retain a new external evidence packet<=128MiB plus8GiB headroom; all cache,
Cargo/Rustup/resolved target/tmp physically on KIOXIA. Preserve but exclude
mutable .remember application state from immutable evidence. Root performs
one fresh whole-feature review, adjudicates private integration, and separately
qualifies immutable native/source then real-workflow measurements before any
performance/release claim. Existing negative gates and BED input-policy holds
remain unchanged.

## Actual candidate and private integration

Owner223e342f845bdb280bd369395483f8f72a1d445f integrates exactly3 authorized
files,28 insertions/31 deletions; index/worktree clean afterward. Offline Cargo
resolution changes only sha2/digest/block-buffer/crypto-common/cpufeatures,
adds hybrid-array and removes generic-array/version_check. Shared cfg-if,
libc and typenum are unchanged. All9 closure archives match lock checksums,
declare MIT OR Apache-2.0 and MSRV at most1.88; actual compiler is1.91.0/f8297e351.
No missing cache, network download or compiler fallback occurs.

The initial real offline build failsE0277 because the new fixed Array output
lacks LowerHex. Preserve that compile error, not a fabricated behavioral RED.
The permitted transaction API adaptation uses one64-capacity String and standard
byte02x formatting; its short String-write expect covers an infallible sink,
not I/O. Open/read errors,64KiB streaming/update/finalize, hash/manifest schema,
file/directory syncs, worker and generation contracts remain unchanged.

Final offline build/fmt/strict all-target/all-feature Clippy pass. Complete
debug/release each17-library+5-workflow executions, zero failed/ignored; each
full CLI graph checks74878 raw cells. Existing consumers/fixtures are unchanged.
An independent data-only replay checks72 actual produced manifests/216 digest
fields against Python SHA-256,64 lowercase characters including600 bytes that
need zero padding; no extra CLI/scientific calls. Root repeats that data replay,
verifies all19 explicit immutable pins, exact3-file patch/current source hashes
and all9 locked archives/manifest MSRVs/licenses. Mutable .remember state is
preserved and excluded, not silently resealed as experimental evidence.

Packet plink-sha2-backend-20261103.S8w3WtDd, report SHA-256
4369ec3fc2218b8dde4efa23d671bf1aaeb04273331b5c7ab4b63159eefbfd05;
PINS63e188a50ee8e45df9538885fc7f4a85cce9227ff0c2b20adf0f603547b1ad86;
delta03305007adb6c306c13e49b48be1e86e65c79026f186d0af7720de3ac957ebf7.
Frozen Cargo.toml10f0528b8be4b84bbe33c0a422b1679f441d17f4a197eeeeafd7487de9ed96d4;
Cargo.lock94cf69d2651deecf518c59ac3c29f265e71a95c270b413dac03bf92ec5a45574;
transaction.rs7cc960a5aa49b921e89ff4eab83e16a08c246c3b53638b589106f405b51eb31d.

ONE fresh whole-feature review finds no critical/important/minor issue; root
reads the entire report, SHA-256
1841983092938940f801cdcb1f44d99ddaad3762dea56a0922722cedf3044eeb.
Final test results are independently verified before integration. Its declined
native/source/provider/performance/publication/cryptographic-internals scope
remains separate: this is normal external-library API maintenance, not a new
cryptographic audit, security claim or performance win. No extra fix/review loop.

## Green16 preparation adoption

Preparation packet plink-green16-preparation-20261103.CRCJ6Xhd retains expected
missing-selector RED and the existing3 preparation tests passing, exit0.
Root reads full minimal selector/test delta, report/admission/snapshot README,
rehashes all frozen prep inputs and independently verifies307 exact committed
source bodies/modes,304 unchanged versus270caf. Only Cargo.toml/Cargo.lock/
transaction.rs differ. Regular sorted USTAR/gzipmtime0 and clean owner/tree
f0596e2476ebe2c53d61cc2c70d4c2e002c3f445 are verified.

Snapshot source559714B SHA-256
38d5aff557934b7b738a9347d00a75f62e6cc8b5a8077c25a8fffba2cb15858a;
files manifest6e841253a0accfd33d720a4d3c8b6fd0958280eb0ff2534651c955db0aeac11f;
prep report44776b58e230fdefc921fb05de06509297dee7b604418276e56e07dff6b24a5f.
Workflow/runtime/scientific policy and17+5/123-report/66-worker/74878-cell
populations are unchanged. Commit and exact-head control CI precede any
separate native/source dispatch. No timing, archive reuse as qualification,
public remote or publication is authorized by capture.

## Green16 once-only dispatch admission

Root confirms remote main517cc538a9f06babc0c543d916408409caa52dc2 and its
exact-head Control plane run41558221553 completed successfully. The current
PLINK workflow list has no same-head run. Admit one workflow_dispatch of
plink-candidate.yml at main with snapshot=green-16, binding this controller and
owner223e342f845bdb280bd369395483f8f72a1d445f. No automatic retry, performance
campaign or publication follows from dispatch. Retain every native/source
original and inspect actual job outcomes before adopting qualification.

Actual dispatch returns run41558979368, attempt1, controller517cc538 above;
the immediate independent run-list read reports queued with that exact head.
No qualification or timing result is available at this admission.

The dispatch-record commit33c51dae322fb0e5cccef6102f1dbb398b82a96b is pushed;
its exact-head Control plane run41559024171 completes SUCCESS. Green16 itself
remains a separate live run, not implied successful by this control-plane gate.

## Comparison draft, not runtime admission

Root fully reads comparison.py, its minimal delta from the accepted arena
runner, and PLAN.json in external packet
plink-sha2-ci-comparison-prep-20261103.3vAOdSTV. Draft runner SHA-256
d8be24faa8fb66de05dbf273a02fd337619bff91c0293418413d284de659c18c;
draft plan5aa27c7c9baf96215d5f993fa57c41142c2097836edf9376dbfd10f74aa4457a.
It retains the existing12-call paired workflow/input/output/resource contract,
uses qualified270caf as baseline, and leaves new artifact/body/budget pins
explicitly unresolved. A main-entry assertion rejects the unresolved candidate
before importing the remote guard or writing remotely. No staging payload,
SSH, provider proof or scientific call is admitted by this draft read. Final
actual CI originals, root adoption, resolved pins and a separate bounded
admission remain required; do not fabricate a candidate or run an old binary.

Final draft handoff leaves those core hashes unchanged. Root reads the complete
README, all12 proposed COMMANDS and STATIC-CHECK, then independently verifies
all16 explicit PINS entries plus PINS.json SHA-256
c0b85d97b7084c4940c91900c6ba2713ee5a26c9a8950f6da1f4403eb03a709c.
Candidate body and stage payload remain absent. Local data/AST scope checks
are not target execution or qualification. All .remember application state is
preserved and excluded, not treated as an experimental identity.

## Green16 native/source originals adopted

Run41558979368/1 completes SUCCESS on all four native jobs and the dependent
same-attempt source job without retry. Root reads the complete report, current
acquire/verify wrappers, all172 lines of the reused data verifier and explicit
adaptations. Root independently replays the full adapted verifier with saves
replaced by comparisons: three existing receipts compare unchanged, excluding
only volatile final_guard after a fresh real physical guard passes. No output
is overwritten and no scientific/native executable runs locally.

Root also rehashes all11093 explicit packet files/modes and four external
recipe inputs. All307 source bodies and committed modes, four native debug/
release17+5 executions,36 generations/74878 raw cells/246 reports/66 workers,
and source224 report plus36 raw bridge calls pass. Their source binding is
this attempt's actual Linux TAR/manifest, not an older candidate.

Canonical adoption is `.autopilot/oracles/plink-native-2026-11-03-green-16.json`.
Original proposal remains root_adopted=false. Packet
plink-green16-originals-20261103.QIB90hqr retains five ZIPs/TARs,10923 original
bodies and all five raw logs. PINS SHA-256
603b3d4c2b8b132b4609d880530b3f92d02ea3ecbcaf3d128695b7f95d3dd71b;
proposal4a7783938f00e59d1f922c66ab1b38b6bec1c6c4f659780635881582837caa68;
original verificationa04fe9cd8995373512ded116ed797447e9115091bcaefe991e96dc8027a393bb.
Linux artifact13196655821 release3339856B SHA-256
4b91395ab024a86c408d216cbd26d30b2ebe1f24c2ff94415b8b572b4aa4c4d8.

This adopts compatibility only. Official runtime/source correspondence remains
unproven; target-host providers, new-binary matched performance and publication
remain separate. Earlier negative whole-performance and no-material-arena-RSS
results remain immutable. No registry retry or performance command occurs.
