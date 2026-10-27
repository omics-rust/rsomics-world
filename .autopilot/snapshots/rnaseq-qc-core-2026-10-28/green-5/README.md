# Source-local RNA-seq pileup contexts

Owning local main59f50135425702ce10f9dbc38fd8dc113b5b5fd6 remains private
and publish=false. The exact pileup dependency is93986e17f4c3a62716b0b6434737c3f46368478a,
version0.9.1, verified by four-native producer CI40588249664. Registry upload
is blocked by authentication; this Git pin is for candidate validation only.

RSeQC5.0.5 restarts pileup state per source query. This product preserves that
contract over one BAM traversal with shared original records, bounded live
span/batch buffering and deterministic parallel replay. Canonical retains its
global pileup path. No new foundation API or path/product dependency exists.

Eleven tests, strict Clippy, formatting and small actual-source single/paired
comparisons pass locally. A fresh integrated review's placed-unmapped sweep
issue is fixed; scoped follow-up passes. The final complete-input1/4-worker
comparison matches all9,810,963 records,58,384 TIN rows and100 coverage bins;
reports match bytewise. No workload, source output or tolerance was changed.
The checked Mac binary is2387e126d4a708c23786edf37f601d3171d4fcdd70c695b87e96181d15b304a5.

Run the ordinary four-native candidate gate, not bootstrap. This is not
formal complete-report CPU/RSS/I/O evidence or permission to publish the
product. Depth-8000 saturation remains uncertified. Historical green4's
chr21-only advantage cannot substitute for this candidate's whole-input gate.
