# BED makewindows source inventory

Read-only inventory; no native implementation, executable witness or performance
acceptance. Root reads the complete upstream core/main and bundled tests, the
historical library, and independently checks the identities below.

## Sources

BEDTools revision705ccfdf2c9a77d71560c8adcece0663c2f5e18e archive:
`/Volumes/KIOXIA/Developments/tmp/peak-input-runtime-20261031.LSP4R2/metadata/bedtools-2.31.1-complete.tar.gz`
SHA2561db4e57c1899aede13ef516c61c9b5819cf2790994a556dbb28972488f9946e6.
Core `src/windowMaker/windowMaker.cpp`:
093b8bcd960523ff5c05b84d548298b5a78ef73daef1844d765b4f12497117c5.
Main `src/windowMaker/windowMakerMain.cpp`:
d0c2be1e88a41477f80d43b8f1cb58ec2ae9c98d7421377c3be266e05024cd71.
The official documentation page contains only a heading; actual source and
bundled test outputs, not the rounded CLI examples, define partition behavior.

Historical clean asset `rsomics-bed-makewindows` at
e3d7050be07ff3e227b96fda150bfc338abe5559; library SHA256
f70820a041a794a0dba43b07585b6b846f872fbcf2b6e23cca7639a0addb4a15.
Only genome-width/step BED3 exists. Retain ordinary tiling invariants and four
oracle goldens. Refactor before merging any core: unchecked additions, its
parser/output shell, and absent BED/count/naming behavior cannot ship unchanged.
Old tiny/subprocess and synthetic Vec benchmarks do not qualify this product.

## Recovered operation

Exactly one genome/BED source and one width/count mode. Width accepts an
optional step (default width), allows overlap or gaps, and clips terminal
windows. Count uses floor(length/count), with the entire remainder in the last
window; bundled tests contradict the rounded-up CLI examples. Count exceeding
length explicitly warns and skips. Reverse changes numbering, never row order.
Names are absent, source, window number, or source plus window number; numbering
resets per source interval. Genome names are chromosome names; BED3 source name
is empty. Source order is retained. No strand or BED12-block interpretation,
fractional sizes, or header-output mode appears in this operation.

The existing product BED/genome readers, record names, ordered chromosomes,
CLI argument selection, common/help, alias checks and transactional output are
the concrete reuse sites. No new foundation, index or depth buffer is needed.

## Initial static findings

Upstream point expansion is printed without restoration. Origin-point count
may emit a negative start; width converts the start to uint32 and can diverge
or wrap. Numeric parsing/zero validation is incomplete. These are static source
findings, not witnessed defects or an approved repair policy. Do not infer
permission from map's specific repair or resolve flank's separate held policy.
Before implementation, capture a bounded whole-operation witness including
ordinary width/count/names and the point boundary; retain actual outputs.

Upstream genome input accepts extra FAI columns; the current product genome
reader intentionally requires two columns. Record that existing format boundary
explicitly rather than quietly changing a parser shared by other operations.

## Six actual source workflows

Packet `/Volumes/KIOXIA/Developments/tmp/bed-makewindows-witness-20261103.hnAJFc7r`.
Root reads the complete3786-byte capture helper and rehashes its inputs before
one guarded execution. Helper SHA256
92d5d1e9bfd4af43d613f03d8b876d08d6e947c3a8ec4c160ccef77a9838b2e8.
Actual guard verifies physical external disk12s1, all Cargo/Rustup/target/cache/
scratch paths and28GiB free. Source is the existing read-only Homebrew2.31.1
binary1691808B SHA256
fc994e1fcfdce8ce8591630d71f21fe67e2275c2a8d967dc0e3dfb4cfa4677a3.
Each call has10s/32KiB-per-stream limits and core dumps disabled; no zero
parameters, large-coordinate probes, retry, download or native build occurs.

All six calls exit0 without timeout: genome overlap/reverse numbering; BED gap
windows/source IDs; fixed count/reverse combined IDs; fixed count/source IDs;
point count; point width. Root reads every full stdout/stderr and independently
replays every helper/input/tool/output hash and status. The length2 interval
warns and skips in both count workflows, exactly as the bundled source test.
Ordinary remainder, ordering, strand independence and names match source code.

The origin defect is now observed: point count emits chrA:-1–0 and0–1, whereas
width omits the origin point. Positive point5–5 emits4–5 and5–6 in both modes.
The source originals remain immutable. An async human decision is requested:
repair origin windows by clipping negative coordinates and dropping empty
results (possibly fewer than requested), or explicitly reject affected origin
input. That affected policy is held; map/flank answers are not substituted.
ORACLE.json SHA256
fbe8319a71fc9a37b0bb9a03a493f61ca929711e12845e5ada1716ed39f42493.
No native behavior, performance or implementation completion is asserted.
