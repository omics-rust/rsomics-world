# DESeq complete real-workflow performance gate

## Current accepted green5 measurement

Fresh run40614312003 at exact12d615548c52092a7f325e8138cd75e85a1f9e0e
passes all unchanged strict source-inner throughput, process-RSS and worker
scaling gates for owningc9f99b7/green5 only. Original artifact/API/log/controller
and2900 files are retained/verified; all16 full comparisons,81 process records,
80 output manifests and244 immutability monitors pass. No measurement was rerun
during retention. The concurrent schema5/local worktree is not this executable.

Median[min,max] seconds: source whole11.38[11.32,11.45], source inner complete
workflow5.495[5.453,5.540], native1 1.05[1.04,1.05], native4 .80[.80,.80].
Source-inner/native ratios5.23x/6.87x exclude package startup while native still
pays all five CLI processes. Whole-process10.84x/14.23x are separately labelled.
Strict1/4 ranges separate, median1.3125x, not linear scaling. Source/native
process RSS medians870584/35880/36168KiB give95.88% lower serial RSS; runtime/
package memory is included, not a claim about algorithmic memory complexity.
Native4 uses more CPU; output blocks52824 exceed source27184. No I/O win.

Actual new machine is AMD EPYC7763 model, four guest logical CPUs on two
guest-visible cores, fixed affinity0..3, Linuxx86_64/R4.6.1/Bioc3.23/DESeq1.52.0,
warm cache/BLAS1. It is not the old EPYC9V74 VM or old R-inner4.454s. Fit/save
stage medians.52/.28s; four repeated reload/export processes remain.12-.14s.
RDS4965252bytes versus uncompressed JSON20218362bytes, no encoding-size benefit.
All1330 discovery IDs/2240 allzero/5800 missing-padj rows and deterministic
full state/four exports match. Source conditions/iterations remain diagnostic
and unmodified, including its named DataFrame conversion warnings.

Evidence:
/Volumes/Zane's HDD/rsomics-fixtures/evidence/deseq-real-workflow-performance-2026-10-28/run-40614312003/.
Artifact12784822601,965496834bytes, ZIP
10fbf20434dac10053562265ab7d67730deefa2c876771739453b26a648562b1.
verification.json SHA31c6da8517f44c26ac827be5b153e29f3778e270954bfb7eb348fbe06329436e;
REPORT.md SHA010eaa0011aec95226c6de355e3ca608f909a49c9056546b5f9f8927956ace2d.
Accepted native receiptc86fec10... binds binary630431e494e5fcba34eb175c7cfe4a68ff071afd249e29619e2385ab7c061059,
36 source files/schema4/all20 workflows. The local padded-NA audit parser
failure is preserved and corrected only in the read-only verifier. Final
scope/current-code/native4/publication gates remain separate.

## Historical green3 baseline measurement

Run40605899832 at exact22f5927e541087fc86c723594d7b96a74691746f is
SUCCESS. Complete compatibility/determinism and the predeclared process-RSS
alternative pass. Strict analysis-throughput and worker-scaling gates do not
pass. Plan: docs/plans/2026-10-28-deseq-real-workflow-performance.md.

Use immutable accepted schema3/green3 run40602456359 at exact
c598ffaf0c44c6b79b1c362ad6d75b4effd91907, owning8bb14ad. Original four-native
evidence verifies. Linux artifact12778884240 is275786601bytes, ZIP
61d64d72c53989ddee0e54a650764f9ec6c766a78131b2baabaa7d95e1bb87be,
binary5e0c5f7f98e0ff791c5217aa3d3ed8f42b9ec1927f1f0ca43cdd98b1e81ca36e,
lock501544e3615498b7f26c06bb5ea36c4d027b61f7366cce4a11b6efcbfc82e241.
Later mean/low-df worktree/binaries must not replace this measurement target.

The bounded harness ran all original14599genes/seven samples through input read,
fit, initial contrast, durable save/reload and forward/reverse/normalized/VST
exports. Source is exact DESeq2 1.52.0/R4.6.1, default serial; native1/4 pays
all five CLI processes. RDS/JSON and input encodings, source inner/whole-process
timing, native summed wall/CPU/I/O and max process RSS are explicitly labelled.
Fixed warmups and3source/5native timed repetitions retain every output and raw
GNUtime record. All16 complete executions,80 measured output files and81
owned-process manifests pass; original bounds/inputs/flags remain unchanged.
All1330 discovery IDs and5800 missing-adjusted-p rows match source. Actual
source iteration-count differences remain diagnostic, not an exact gate.

Median[min,max] seconds: source whole9.49[9.45,9.57], source inner workflow
4.454[4.447,4.491], native1 4.56[4.45,4.62], native4 4.28[4.26,4.55].
Source/native process RSS is869172/35796/35984KiB median, respectively.
Both strict memory comparisons pass, approximately24.28x/24.15x lower process
RSS including the loaded R runtime, not a statistical-algorithm complexity
claim. Native maxima4.62/4.55 exceed source inner minimum4.447; no strict
compute win. Native1/4 timing ranges overlap; median1.0654x is not scaling
acceptance. Whole-process2.08x/2.22x ratios include R package startup.

Host is AMD EPYC9V74 Linux x86_64, fixed CPUs0-3: four logical CPUs mapped
to two physical cores with SMT. Warm cache; all input-block counters zero.
Native output blocks52824 exceed R27184. R's default compressed RDS is
4965507bytes versus native20218235-byte uncompressed JSON; no I/O-volume or
identical-encoding advantage is asserted. Source inner timing includes input,
fit, initial contrast, full save/reload and all exports, excluding package
loading. Native pays five real CLI processes and repeated bundle loads.

Measured run/read-fit-save dominates: native1 median4.01s, system3.23s;
native4 3.84s, system3.22s. Reload/export commands are about0.10-0.12s.
The earlier bundle-load hypothesis is rejected. A later scoped save fix
reuses published common0.12.3 buffering; its local A/B belongs to a separate
candidate and must not be attributed to this immutable baseline measurement.

Artifact12781050792 is865169279bytes, ZIP SHA
cec5999e9c15465b57c5bd65f510aca1c0b24ba7e75d1afe22db9d527dbe698a.
All2393 original members, API/job logs and exact-head controller are retained:
/Volumes/Zane's HDD/rsomics-fixtures/evidence/deseq-real-workflow-performance-2026-10-28/run-40605899832/.
Authoritative verification.json SHA
9ef2c2af1593a69dd23e2fd1abdfe45d197a6a2f3c340f950dff2cb60ceb242c;
REPORT.md SHA864134508cc8b4581390d23f7921f6feddfb9196a5d7c91bd9a86051ec6cf4e6.
Already-accepted embedded oracle/native archives were bound, not redundantly
re-audited. Native schema4/local-trend scope and registry authorization remain
separate from this resource-only performance acceptance.
