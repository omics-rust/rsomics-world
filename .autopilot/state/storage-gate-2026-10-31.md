# Local storage stop gate

## Resolution: external project storage, no global occupancy gate

The user explicitly authorizes changing this requirement and continuing while
keeping project files off the internal disk. The boot-container80% threshold
below is historical and no longer blocks work. AGENTS.md and the five active
owning guards now verify physical external Cargo home/resolved target/rustup/
toolchain/TMP/cache paths instead; Cargo target/config overrides are rejected.
All five pass bash syntax and guarded --exec true checks. No project output,
shared application state or user data was deleted or migrated. Bounded checks
did not identify task-owned project files requiring cleanup; they are not a
claim of zero Codex/OS footprint. Unrelated total boot usage is out of scope.

One independent guard review identifies two gaps: comparing filesystem IDs
against `/` alone does not establish an external mount, and `--exec` skipped
target/config override rejection. Parent corrects both in one pass: diskutil
must identify the exact KIOXIA mount as Internal=false before any directory
creation/Cargo execution, every checked path belongs to that device, and
override rejection applies through --exec too. A harmless true command with
--target-dir is observed passing before the fix and rejected after it; no
target directory is created. All five final guards pass syntax/external
preflight and reject the override. Generic --exec is explicitly not a sandbox.
Review: /Volumes/KIOXIA/Developments/tmp/storage-guard-review-20261031.uxDMQU/REPORT.md.
Final deseq/ecology/structure guard SHA9ad8dc36e6211ddf66d6510f4e87b6b3cc8873cadf4ad178403691e6c840585b;
edgeR8b13981d6de8a74cf05c1991a70b952a7833d794b42f1441a6d746d1adf27d67;
limma5bf8929cb24b594af505ac96c0a582de234873c03f95253db18a29f91b255060.
Separate owning storage-policy commits: DESeq917ede8, ecologycfb3b8e,
structure58493ba, edgeR8111941 and limmab54ad8c. No analytical source changed
in those commits. edgeR/limma current README policy paragraphs are included;
the other owners' current README updates accompany their ongoing feature/
preparation edits. Historical source snapshots/notices/reports stay immutable.

The goal remains active. Structure resumes its interrupted whole-feature
suite, and DESeq resumes unpacked package tests into new logs. Earlier exit130
and partial outputs remain preserved and are not counted as passes. Ecology's
completed rarefaction evidence/review can now be adopted. Shared live Codex
history/database and OS-managed swap are not ordinary project scratch and
must not be deleted to satisfy this project-storage policy.

## Historical trigger and stopped checkpoint

STOP affected local work pending user decision. Fresh DESeq release-review
guard observed boot APFS occupancy0.8001193971506715 (80.01194%), free
48992174080bytes. Parent independently confirms0.8000922918183473, size
245107195904/free48998817792. This exceeds the explicit80% stop threshold.
All checked Cargo home/resolved target/TMP/toolchain paths remain external;
the cause of boot-container growth is not diagnosed. Do not assume it is
project output or delete any data.

Parent interrupts structure author and asks all agents to stop. Root Cargo
test21916 and structure Cargo23284 receive SIGINT; structure test23905 receives
SIGTERM after exact PID/command validation. Orphan local-trend test24076 exits
before the final SIGTERM attempt (no such process); subsequent scoped pgrep
finds no matching live project build/test process. Root session21741 exits130.
Preserve partial logs/outputs;
interrupted checks are not passing results. No cleanup/deletion, publication
or new build/test/retry is authorized while this stop is unresolved.

Last completed progress:

- World2fe515349eb4cc05c41b3a433971003dadca39a0 passes exact controller
  41177407552. Local world4f393790b675c38d4b2622d78728730fcdf5c5d5 adds
  actual DESeq preparation plan/state but is not pushed yet.
- Structure reviewed I/O committedd7309d563be58e82dd5e943e6372ce29f0607834.
  Real analysis/selector source is now uncommitted; ordinary torsion source
  comparisons and spatial tests pass, whole CLI checks remain interrupted/
  incomplete. Degenerate torsion public adapter is absent pending prior choice.
- Ecology rarefaction code is uncommitted, all four integrated tests, format,
  strict Clippy and release build pass. Frozen binary574b084984291a89600db92f09ec35bb81a3a57eef0a1e470239348a6718721a;
  984captured count checks/594alpha checks pass at1/4 with unchanged bands.
  Independent whole-feature review3cc7fff16aef24c50f60a34f9066e284511ce49989800e443a11df4b543ad81d
  has no findings. Parent has read complete source/tests/comparator/docs/review;
  final implementation report/evidence still need parent adoption before commit.
- DESeq new metadata/README/one about string/CI/examples/dependency notices are
  uncommitted. Initial120-file candidate package9cf2cb17... builds after unpack;
  final unpacked tests/strict checks are interrupted, not accepted. No owning
  GitHub repository has been created and publish=false remains.
- Registry credential remains unchanged, no authentication retry or publication.

Evidence roots remain external:
/Volumes/KIOXIA/Developments/tmp/deseq-release-preparation-20261031.akudyQ/,
/Volumes/KIOXIA/Developments/tmp/ecology-rarefy-contract-20261031.QhESSv/,
/Volumes/KIOXIA/Developments/tmp/structure-native-analysis-contract-20261031.0cVmk4/,
/Volumes/KIOXIA/Developments/tmp/structure-spatial-20261031.hGNvHh/.
The structure author's final stopped checkpoint/tool log may add detail.

The goal is not complete and was not user-paused. This is the first confirmed
storage-blocked goal turn; do not mark blocked immediately or resume the affected
actions by silence. Ask whether the user will free space or wants a read-only
occupancy inventory/cleanup proposal. No deletion without exact scope and
recoverability, and no restarting interrupted work before decision/fresh guard.
