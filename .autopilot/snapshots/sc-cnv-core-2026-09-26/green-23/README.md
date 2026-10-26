# Denoise negative-number CLI correction

Compared with green22, only src/cli.rs and tests/cnv_cli.rs change.
The retained green22 binary rejects --noise-filter -1 at argument parsing,
although the documented/upstream operation treats nonpositive values as a
bypass. Add a real positive/zero/negative invocation regression and enable
negative numbers for this one option.

Numerical library, report schema/implementation and all other source files
are byte-identical to green22. The installed-package comparison's positive
and default cases still use the retained green22 binary. Final four-native
tests exercise this parser correction as well as the whole denoise feature.
No numerical tolerance, old oracle or dependency changes.
