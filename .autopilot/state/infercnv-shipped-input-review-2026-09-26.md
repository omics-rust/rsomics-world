# Independent shipped-input preflight

Read-only review by `/root/infercnv_oracle_review` of pinned infercnv source
`b421d9405c97a309b081ef86d455e976df93eae4` and its original three shipped
inputs. The reviewer did not read the new generator/checker implementation.
All three original input hashes match the binding design. No R execution
occurred: these are source/data-derived predictions, not accepted goldens.

## Identity predictions

Ordered-ID hashes encode UTF-8 IDs, one per line including terminal LF.

| Case | Stage | Genes | Ordered-gene SHA-256 |
|---|---:|---:|---|
| Subset | 1 | 1775 | `6d4e48f8249c300d3e97064b0b53d8cfc920e62e3cb5f4bd9caf28635c25d898` |
| Subset | 2 | 1487 | `db023a1d6ed642448469665181825a85e1fe5d3d6673bce639b27833403b4f03` |
| Full | 1 | 9939 | `cfba69ab79982ec95a42f885ba1456a4820ca7213886e2dabb7599355179cca8` |
| Full | 2 | 8508 | `e1ff0e46c045a0317b9bc6569dce9cfd149600891c2d59b2d4ebbd7df502f94d` |

Gene order follows chromosome first appearance in coordinates, then start
and stop, not lexical chromosome order. There are no coordinate ties.
Stage 2 preserves retained order. Source: `R/inferCNV.R:401-413` and
`R/inferCNV_ops.R:2154-2159,2182-2184`.

| Chromosome | Stage 1 | Stage 2 | Chromosome | Stage 1 | Stage 2 |
|---|---:|---:|---|---:|---:|
| 1 | 1011 | 852 | 12 | 556 | 472 |
| 2 | 708 | 615 | 13 | 187 | 162 |
| 3 | 607 | 535 | 14 | 351 | 301 |
| 4 | 350 | 288 | 15 | 318 | 274 |
| 5 | 480 | 420 | 16 | 450 | 397 |
| 6 | 533 | 453 | 17 | 628 | 546 |
| 7 | 514 | 458 | 18 | 148 | 126 |
| 8 | 354 | 297 | 19 | 656 | 545 |
| 9 | 410 | 349 | 20 | 283 | 239 |
| 10 | 428 | 363 | 21 | 108 | 90 |
| 11 | 611 | 514 | 22 | 248 | 212 |

Both cases retain all 184 cells in original matrix-header order. Their
ordered-cell SHA-256 is
`7957dda41d16809670557542625eaf9ba25d5fc288e0693e2d68999a4c7e3b85`.

| Columns (one-based) | Group |
|---|---|
| 1-23 | Oligodendrocytes (non-malignant) |
| 24-42 | Microglia/Macrophage |
| 43-75 | malignant_MGH36 |
| 76-109 | malignant_MGH53 |
| 110-149 | malignant_93 |
| 150-184 | malignant_97 |

Reference list order follows requested names: Microglia then
Oligodendrocytes. Observation list order is sorted labels: malignant_93,
malignant_97, malignant_MGH36, malignant_MGH53. Neither list construction
reorders columns; annotation-file order is not matrix order. Source:
`R/inferCNV.R:285-312`.

## Filtering witnesses and limits

| Case / boundary | Gene | Mean over 184 cells | Positive cells | Retained |
|---|---|---:|---:|---|
| Subset below cutoff | DENND1B | 0.9956032663043478 | 39 | No |
| Subset above cutoff | SYNJ1 | 1.0084782934782608 | 56 | Yes |
| Full below cutoff | TMEM17 | 0.9997394402173913 | 103 | No |
| Full above cutoff | PTCHD4 | 1.0007118804347825 | 147 | Yes |
| Both, exact detection boundary | TNFSF18 | 2.701111956521739 | 3 | Yes |
| Subset below detection boundary | IGFL4 | 0.5291032608695652 | 2 | No |
| Full below detection boundary | GNGT2 | 0.723016304347826 | 2 | No |

Mean filtering removes 288 subset / 1431 full genes. Detection filtering
removes zero additional genes: the last two examples already fail the mean
filter. TNFSF18 exercises inclusion at exactly three detections, but these
real cases do not isolate exclusion by detection count; retain the synthetic
witness. Decimal sums for the nearest full-case cutoff examples are
183.952057 and 184.130986, so neither is a numerical cutoff tie.

Coordinate literals are integers from 14363 through 249214145, within signed
32-bit range. Preserve upstream integer start/stop columns when exporting;
unnecessary conversion to double could introduce scientific notation.
