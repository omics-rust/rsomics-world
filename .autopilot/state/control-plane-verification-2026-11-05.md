# Control-plane verification runtime

Current local verification uses existing Python3.12.11 at
`/opt/homebrew/Caskroom/miniforge/base/bin/python3`, read-only as a tool.
The external-disk guard supplies TMPDIR, XDG caches and
PYTHONDONTWRITEBYTECODE; no Python installation or environment mutation occurs.

The full542-test discovery passes with two defined skips; architecture-ledger
validation also passes. Child monitor cases requiring40GiB free remain skipped
because KIOXIA has about20.6GiB; this is not a boot-disk condition or an excuse
to delete fixtures. Original log:
`/Volumes/KIOXIA/Developments/tmp/fastq-quality-install-20261105.3lXaUJvO/control-tests-python312.log`.

An earlier run mistakenly invoked macOS system `/usr/bin/python3`3.9.6.
It ran422 discovered tests with4 failures,158 errors and1 skip; that log stays
retained as `control-tests.log`. Current control scripts use newer Python
features. Even Counter equality with explicit zero entries changed in3.10,
explaining that interpreter's spurious registry-summary mismatch while actual
rows remain17live/7repo-only/6planned. No registry row or validator logic was
modified to conceal the unsupported-interpreter failure.

The local Python3.12 verification and exact-head hosted Control-plane CI remain
separate gates; do not infer the latter from the former.

Hosted run41965530121 at exact head
`794669677f23afd5ba716af696c803f8b5f36de7` subsequently finishes SUCCESS,
including both full control-plane tests and architecture-ledger validation.
Later plan-only commits must still pass their own exact-head run.
