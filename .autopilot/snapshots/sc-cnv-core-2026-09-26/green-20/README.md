# Integrated canonical group-count test correction

Compared with green19, only tests/cnv_cli.rs changes. Actual green19 compiles
after the lock repair, then its canonical test fails: Microglia/Macrophage
has19 members, while the new test mistakenly expected23. The accepted original
maps.tsv and shipped full01.cells.tsv independently confirm Microglia19,
Oligodendrocytes23 and observations142 in the requested reference order.

Correct that hand-entered reversal and bind each count to its group name.
No production code, old oracle fixture, threshold or numerical tolerance
changes. Earlier real assertions already pass8508 retained genes/184 cells,
6187 selected/2321 outlier genes. Full CLI1/4-worker equality and the final
four-native integrated gate remain pending, not inferred.
