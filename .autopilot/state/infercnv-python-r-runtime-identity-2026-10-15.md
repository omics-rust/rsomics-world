# Python identity across R launch

Plan: `../../docs/plans/2026-10-15-infercnv-python-r-runtime-identity.md`.
Base world56e70d79643af9408f0c5bd556e8a60276d725d9.
Status: green factory candidate preserved; actual runtime/library RED proved.
Native interpreter selection applied; hosted boundary/factory GREEN pending.

Original run38696325035 succeeds, artifact11954931733,34000792 bytes, SHA256
`02b4ea3592c23a50b5c1e3d98d95eefd0d59b222817eaddf9c19539e15bd29d9`.
Full log ZIP SHA256:
`13beae660e67e327f2ac2cfcc1dce2c72c67b469e5e3c3058f8004b5949f421d`.
Root authenticates original run/job/required steps/API/listing/size/digest/CRC
and277 safe unique regular ZIP entries before external extraction. All original
evidence stays under run-38696325035, never overwritten.

Independent local exact checks on both parent plain/gzip and all4 fresh warm/
measured states pass:9939 genes184 cells, count bits/identities/coordinates/
roles/ordered maps and literal12 counters. Source input hashes are checked
against the accepted input receipt before using original stage1/maps. This
proves these values/counters, not final runtime identity or performance.

Audit discovers outside-R Python3.12.15 while both R-child records report
Python3.12.3 at the toolcache3.12.15 executable path with the same executable
SHA256. Both satisfy the weaker3.12-family check, but exe hashes alone do not
prove which libpython executed. Observed R threads are2 before/after despite
seven requested limits1; no actually-single-thread or factory-peak claim.

Ruling: withhold final runtime/paired acceptance, preserve green candidate and
add actual cross-process identity regression before dependency installation.
Compare canonical executable/hash, exact sys.version, Python prefixes/stdlib
path and mapped libpython paths/hashes, not only directory names or --version.
Actual hosted R ldpaths/LD_LIBRARY_PATH are logged without credentials.

Hypothesis: R prepends system lib directory ahead of toolcache library. Existing
4090 R4.6.1 /etc/R/ldpaths confirms that mechanism, but hosted diagnostics must
prove the actual conflict. If proved, use native Ubuntu Python3.12 instead of
altering R's loader environment or wrapping a binary and mislabeling its hash.
Trial/package/helper/checker sources and all12 pins remain unchanged.

Move existing3 executable baseline capture from after package prerequisites to
the new after-R-setup boundary; preserve both the later pre-factory recheck
and always-postcheck. This makes the baseline cover all project R execution
and lets an intended early runtime rejection retain complete postchecks rather
than fail secondarily because the downstream baseline was never captured.
No baseline is overwritten or R loader environment modified.

Root validates complete embedded workflow equality, all29 actual Bash blocks,
actual Python diagnostic syntax,283 controller tests, architecture/whitespace.
No actual hosted identity RED yet; native interpreter selection is still only
planned, and the toolcache action is unchanged until that RED is observed.

## Hosted runtime/library RED proved

Commit539ab57706c0620f23699414fc17a1b96faf60ac passes exact-head Control
plane38698956363. Installed-mode run38699064898 at that head fails ONLY the
new Python/R boundary. Original raw artifact/input and R setup succeed; all12
code and3 executable postchecks pass. Dependencies/factory steps are skipped.
Root authenticates original APIs/head/attempt/job/size/digest/CRC and23 unique
safe regular entries, exact diagnostic and actual mapped-library records.

The same executable SHA53dab24b...a675e loads toolcache libpython3.12.15 from
`/opt/hostedtoolcache/Python/3.12.15/x64/lib/libpython3.12.so.1.0` outside R,
SHA c7c1a2206db0fa089f8a252d1652d5ccb82669774dbb599427082ffc9eaaae85;
inside R it loads `/usr/lib/x86_64-linux-gnu/libpython3.12.so.1.0`, version3.12.3,
SHA1021f236227334ded6d1bbbc4bb6f5d933f793d8397c969896cfbdf03ce96223.
Actual hosted R ldpaths/LD_LIBRARY_PATH proves system directories precede the
toolcache library. This confirms the hypothesis, not a guessed label fix.

Artifact11954419361:9278 bytes, SHA256
`ed12d9a095b8378f52465cfcc052a9ed2a3e022c114b6a217dac2f822825d598`.
Full logs SHA256:
`1af420a833f309353e2035ed228f54102d16a46067587deddd67b7db9c36d318`.
All originals retained externally under run-38699064898.

Only after that observed RED, replace setup-python action with explicit native
`/usr/bin/python3.12` selection and a PATH-only symlink directory. No R loader,
R source, timer, assertion, checker or12 fixed pins change. The exact runtime
probe remains unchanged and must pass before full factory execution.

Native-selection candidate passes all283 controller tests, complete embedded
workflow equality, all30 actual Bash blocks, unchanged12 code/input pins and
architecture/whitespace. Actual native boundary/factory run remains pending;
fresh whole-component workflow review runs independently, without build/R.
