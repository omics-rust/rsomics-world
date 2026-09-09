# Reheader extended BGZF read-path repair candidate

Full four-native source snapshot, not a product release. All 174 entries and
hashes were verified. Exactly four paths differ from the expected-red
snapshot: `src/format/bgzf.rs`, `src/reheader.rs`, `src/reheader/bcf.rs` and
`src/reheader/vcf.rs`. The complete 19-group reheader CLI test is unchanged.

Expected-red run `34293512685`, world
`540a8094ef6b724a074f722cec4c9473bc8d2462`, fails the reheader step on all
four native targets; control CI `34293413259` passed. The Linux ARM raw log
was read before production edits: 18 groups pass and the new 36-case group
fails because every legal extended BGZF input is misclassified as ordinary
gzip. The full artifact audit is separate from that observed assertion.

The classifier now delegates extra-field structure to the existing private
frame parser. A bounded single-member gzip decoder accepts arbitrary legal
subfields, checks CRC/ISIZE and rejects data after the member within a block.
It allows at most 65,536 inflated bytes. BCF reheader consumes this decoded
reader directly, avoiding Noodles BGZF 0.49's fixed-18-byte-header decoder;
VCF header inflation shares the same helper. Neither dependencies nor public
APIs change. Six new unit groups cover decoded size limits, forged ISIZE,
CRC/ISIZE corruption, extra member bytes, short reads/empty frames and terminal
EOF. Those unit groups are additional coverage, not observed-red evidence.

The [pinned BGZF specification](https://github.com/samtools/hts-specs/blob/da617203a9527537746e200abda2885bec3a822c/SAMv1.tex#L846-L905)
allows the tested extra subfields and limits both compressed and inflated
blocks to 64 KiB. The unchanged frame parser enforces compressed size and the
product's stronger single terminal canonical EOF contract. VCF's unchanged
raw tail copy performs structural validation, not full payload CRC checks.
BCF terminal EOF validation requires consuming the reader. Potential early
logical EOF and magic bytes split across tiny initial frames need separate
regressions; this candidate makes no broader BGZF support claim.

Independent source review found no blocker for remote execution. Replacing
Noodles' reusable decoded buffer with this per-frame allocation/decoder needs
representative performance measurement; no no-regression claim is made.
Rustfmt/diff and control-plane static checks pass; Rust tests have not run on
this repair snapshot. Dispatch with `regressions_only=false`.

| Artifact | SHA-256 |
|---|---|
| `source.tar.gz` (334569 bytes) | `6ebf29ca9842bd16279be21ef30d60a7c514f01ef08ce273743509e9eda6551e` |
| `files.sha256` | `2854472a672998e56e52653b50755505974bacccd3dc4f63382302e415f4f32e` |
| `tracked.patch` | `253899e8f71dd2090c858b1127ea5afe9aabe8a01c06dc173e427d4280eeae6d` |
| `tests/reheader_cli.rs` | `d58d168be68f3725af43cf0b97a2534d1545ab07dcf11643de154f787eb69db4` |

Owning product HEAD/index remain unchanged at
`682942cfa69768dc3a127a8544f2f07213b704ea`. Boot APFS occupancy is 95.06%;
no local Rust/product execution occurred. Scratch and artifacts are external.
No publication or performance gate is closed.
