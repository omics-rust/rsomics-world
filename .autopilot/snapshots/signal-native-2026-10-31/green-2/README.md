# Signal native candidate with source-profile header sum

Owning repository: `/Volumes/KIOXIA/Documents/omics-rust/rsomics-signal`.
Owning main commit: `0f4690363d8dc29b498fceeb07605b855d400032`.
This real38-file product remains private and publish=false; no release or
native/source acceptance is inferred from freezing it.

Source archive SHA256:
`946334c1000eb1e9622ed32b2772346b3dd60a8ee338e175af2ba24178155b06`.
Source manifest SHA256:
`4f1ce7d5dd04309ca5c4d148772b57dda57541a9b87da4668141b677f62ac6f3`.
Cargo.lock SHA256:
`d2770d56b6295dd5ab10d39f3a42ab3a8ef6d69bdd1c9e079f661a96d53736be`.

Actual green-1 run41212042509 passed all four native lanes but failed the
Linux same-native-file source gate on sampleB total-header sumData.
Retained raw headers and pinned source establish f32(span*value) followed
by f64 addition, rather than the underlying codec's f64 product.
Only this private summary scalar is corrected. Runs, zooms, query arithmetic,
other header fields, source runtime and numeric bands remain unchanged.

The existing native-spine test first failed with161290.400390625 instead
of161290.40625, then passed after the correction. Its full-resolution mean
still uses the unrounded sum. The complete existing suite passes10 library
tests and1 connected six-operation test; formatting and strict all-target
Clippy pass. Only the two affected source files change from green-1.

Correction packet:
`/Volumes/KIOXIA/Developments/tmp/signal-header-sum-correction-20261031.wD8EAd/`.
The original failed run and green-1 snapshot remain immutable. The approved
three upstream defect repairs remain adopted; optional sortedBED policy is
still pending. A genuinely new four-native/same-file source gate is required.
Representative performance and public delivery remain separate gates.
