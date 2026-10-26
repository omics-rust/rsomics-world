# Column-parallel smoothing candidate

Extends the CLI's existing worker selection into triangular preprocessing.
Each worker owns complete output cell columns and reads immutable input.
Neighbor accumulation order, chromosome boundaries and numerical budgets
remain unchanged. Finite normalization/result checks run once per output
gene instead of inside each neighbor iteration.

The product-local worker dispatcher now has two actual consumers: matrix
smoothing and condensed distances. It joins all spawned workers on errors.
No public foundation, dependency, CLI operation or algorithm is added.

Existing configuration callers select one worker explicitly. Meaningful
new checks cover bitwise smoothing equality across worker counts and
unchanged input on arithmetic failure. The existing canonical raw CLI
comparison now includes all expression values, not only selection/trees.
Old oracles and tolerances are unchanged.

This candidate is not accepted or published. First run the existing Linux
correctness/benchmark mode. A useful measurement, one overall review and
the four-native debug/release gate are required before the product commit.
