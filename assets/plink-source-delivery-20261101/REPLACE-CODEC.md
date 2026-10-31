# Replace the LGPL codec and rebuild

Keep the original verified delivery unchanged and make a separate working copy.
Edit vendor/pgenlib and/or the LGPL adapter under native/ within its code/ tree,
retaining license/copyright notices and marking modified files and dates.
The interface to preserve is native/pgen_shim.h ABI2. No product command,
organization signing key, online approval or original-library source hash
allowlist is needed for an ABI-compatible user's rebuild.

Build the modified working copy with the same ordinary offline Cargo command
and independent build directories described in REBUILD.md. Do not alter the
original delivery manifest or registry-package .cargo-checksum.json to label
the derivative as an original release. Record its changes and new identities.
Preserve applicable component terms, debugging/relinking rights and installation
information when redistributing; these instructions are not legal clearance.

The private hosted delivery proof makes one controlled modification to the real
PgrInit reader body: it reads a local volatile marker while keeping ABI/data/
results unchanged. Reader object, codec static archive and final executable
must all change and contain that marker, unlike the baseline. Both versions
must pass the same existing complete1/4-worker CLI workflow and raw-state checks.
That marker is a test of actual replacement, not a new production feature,
required user patch, scientific benchmark or reason to weaken source integrity.
