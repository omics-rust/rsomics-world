# Completed installed Bayesian witness

Actual corrected Bayesian-only run40499067515/job122171875675 succeeds on
world9dd9dfc6e96fb9358a4639fbf9072679c2178db0. It reuses accepted grouped-HMM
replay, not a newly repeated raw HMM campaign. Both actual public sampling,
filtering and report-adjustment calls preserve original expression/states.

Original API/run/job log/ZIP and verified extraction are retained under
/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-bayes-2026-10-27/
run-40499067515. Artifact12735046764 is902489696bytes, SHA
2fa6b153ea89b848f4edff7b4efc84227df3930647acbd7845e25463ca2c3dbe.
The normal successful-artifact verifier passes, including API/head/status,
ZIP safety/CRC, digest and shipped manifest. The Bayesian receipt is
.autopilot/oracles/infercnv-bayes-workflow-2026-10-27.json. Shipped manifest
342d1a68c294443ab9b422ef7cff88f3d8d7cc68d9f6b971ec9ac9e2ffbdcb56 is unchanged.
Bayesian witness SHA is
69ca75877f3960ed4b8b20b26bb2db14cfe92357ee7d4c069fed9e5f77397fb6.

## Actual results

Both repetitions have82 regions, remove only chr7-region_165, and reassign16
retained regions. There are no removal or final-state differences between
these two actual repetitions. All492 theta means differ numerically and the
first region's raw samples differ; these are separately executed stochastic
calls, not duplicated records. Largest observed theta difference is0.0030003831
at chr1-region_105/state2,0.378145 of the declared combined-MCSE tolerance.
Maximum per-cell probability difference is0.018.

Each actual uncertainty reduction agrees with its original posterior table.
Both classify the same three unresolved state3/4 maxima: chr3-region_59,
chr5-region_161 and chr16-region_182. Current choices are4/3/3 respectively.
Their matching choices here do not erase the measured uncertainty or imply
JAGS/native bitwise identity. Native comparison uses the predeclared statistical
contract and explicitly labels any admissible reassignment difference.

R4.6.1/infercnv1.28.0, rjags4.17, JAGS4.3.2, coda0.19.4.1, localeC.
Six500-adapt/200-update/1000-kept JAGS chains; R seed261027, no explicit JAGS
seed in source. Actual in-process sampling/public-call walls407.950/430.101s;
filtering3.303/3.260s. These include trace/evidence work and are not paired
native speed/RSS evidence, convergence proof or biological validation.

## Next actual gate

Run green27, not green26, on all four native targets with bayes_workflows=true,
oracle_receipt=none and all unrelated campaign flags false. Current private
product integration remains uncompiled/unaccepted until this runs. It must
preserve original raw expression/HMM outputs, posterior/cell joins, exact
stable state/structural reporting and indexed1/4-worker reproducibility.
Do not repeat the accepted denoise/grouped-HMM or closed failed collector runs.
The single fresh feature review/fix pass is recorded separately; no second
per-helper review is required. Performance and broader inferCNV/portfolio work
remain open. Ward policy awaits its separate user decision.
