# ASCII-padded numeric raw-reader candidate: green-10

Retains the verified gzip lock and all green-9 gates. Actual four-native
run 36694720861 exposed ASCII numerical padding from the immutable accepted
R formatC inputs. Only count tokens now trim ASCII spaces at their ends;
identity fields and coordinates remain literal. No rounding/tolerance change.

Five focused regressions increase ordinary input tests from 26 to 31: padded
ordinary/scientific/native decimals, literal spaced identities, rejected
Unicode/control/tab padding, spaces-only values and unchanged coordinates.
The existing malformed discarded-row samples replace newly valid ASCII
padding with spaces-only and Unicode-invalid tokens.

Three external tests retain actual synthetic and shipped raw-to-checkpoint
comparisons. Four-native debug/release, measurement tests, bench compilation
and strict Clippy remain required. No ingestion-speed, default inferCNV,
downstream, CLI or release claim is made.
