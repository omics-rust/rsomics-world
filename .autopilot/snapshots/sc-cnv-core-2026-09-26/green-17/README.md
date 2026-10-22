# Private native samples gene-filter candidate

Candidate working source based on productce64fd41. Compared with red-11,
only src/cnv/samples/filter.rs and distance.rs read-only accessor wiring
change. All tests, fixture expectations, Cargo metadata and lock remain
unchanged. The new filter is product-local and is not yet a library export.

The source-ordered f64 implementation preserves reference order, the fixed
positive-guard threshold0.8, inactive NULL versus active empty outliers and
R's empty-negative-index behavior. Unsupported nonfinite reductions,
nonconstant zero-variance underflow and active <=1 elements fail explicitly.
Requested index-buffer peak is max(R, active?2G:G)*sizeof(usize), with
validation scratch released before output allocation. No reference-matrix
copy, epsilon, dependency or speculative foundation is added.

Actual native RED39801195710 confirmed missing filter.rs after source/oracle
and target-path admission. Existing11 filter contracts plus all52 original
route selections, filtered value/order checks and152 composed distance/Ward
trees run through the existing four-native debug/release candidate workflow.
GREEN is pending, not implied by this snapshot or local formatter success.
No group-owner, CLI, full inferCNV, speed or release claim is made.
