# PLINK complete output blocks

Status: implementation started, not integrated or qualified.
Plan: `docs/plans/2026-11-03-plink-complete-output-blocks.md`.
Owner base:499693cf5629420a0a5171e700afdda264e3aa0d.

This follows source-confirmed job-internal blocking, not measured causal
attribution. The chosen private protocol transfers one completed payload per
lane and waits for writer return before reusing it. LD output jobs have an
admitted4MiB cap, a derived line bound and serial fallback for over-cap lines;
make migrates its existing completed blocks. Prune is independent/unchanged.
No larger queue, new public component, disk spool or numerical change.

One bounded implementer owns the complete source change and its tests/logs.
Root owns the complete review, independent verification and commit. New source
requires a new immutable/native gate; ongoing metadata green11 run41478980090
stays bound to499693cf and must not be represented as qualifying this work.
All external-device guards, preservation and exact-head gates remain required.

The full30-product goal remains active. This is a performance-relevant product
implementation wave, not a redefinition of portfolio completion.
