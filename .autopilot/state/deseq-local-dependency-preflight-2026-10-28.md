# Local-fit dependency qualification

## Current decision

The Rust candidate below remains unadopted; its opaque fitted state still
needs a real no-refit persistence solution. Official locfit1.5-9.12 source is
now qualified at https://cran.r-project.org/src/contrib/locfit_1.5-9.12.tar.gz:
207112bytes, SHAfa31b9d407f3557a6fcb7105cbd7b16a32a6fb868a7c60bc38a3e33cb7a39343.
MD5358f4fde91d135b36ec2a2331d75393b matches CRAN PACKAGES. The analogous
Archive URL is404; use the verified release URL/hash. Actual R member is
locfit/R/locfit.r, not a capitalized .R filename.

Parent read the complete official-source report and actual formula/default/
returned-state/prediction code. Full report:
/Volumes/KIOXIA/Developments/tmp/locfit-official-source-20261028.pWLEjE/REPORT.md.
The old oracle records installed version/library root, not the loaded locfit
DLL hash; do not retroactively claim binary authentication. A bounded four-case
source workflow will record actual runtime identity, frozen inputs/queries,
local-VST/FMM state and one fresh-process no-refit reload. Parametric fallback
must actually occur or remain uncovered; no seed search is authorized.

Mean/low-df native work is now implemented separately, not pending a new RNG
dependency: see deseq-mean-low-df-native-2026-10-28.md. It uses its qualified
fixed-reference profile plus real runtime observations, not an unimplemented
general R RNG. No new locfit foundation or public API is justified.

## Initial candidate inspection

A bounded read-only qualification identifies rcompat-locfit0.1.5 as a candidate,
not an adopted or certified dependency. Official registry reports non-yanked,
MIT, MSRV1.70, ownerdeminden and no runtime dependencies. Streamed169033-byte
archive SHA57fc3c85ee6ee951198c4e105220025963815d8d981af873625044877b458572
matches registry. Published VCSd1fe58bba1cfd8aded7fdd91ab0350b8f65bce5f,
treee662e48b99fd832efd2f3c8d3b867e8ec89e929c; all seven src blobs match.
No package installation, source edit or adoption is performed. Archive bytes
are inspected in memory after the external/APFS guard, not saved on boot disk.

Actual implementation includes nearest-neighbor/tricube weighted local
polynomials, adaptive grid, Hermite approximation and extrapolation. Its
DESeq wrapper uses log dispersion/log mean with mean weights and the proper
minDisp*10 filter. But production still has probe-driven n=5/7,10,30 and
100..999 tie/order/FMA branches. No geneID/fixture output lookup is found;
that does not prove general locfit compatibility. The declared clean-room/MIT
status is an author statement/license, not independent legal certification.

Existing tests include an actual1,198gene DESeq local-trend case, not the whole
fit/VST lifecycle or four-native evidence. Some external diagnostics are ignored.
The fitted object has no serde or knot-export API. Any adoption must satisfy
our no-refit saved-workflow contract, not assume opaque model state is durable.
Its Hermite approximation is not R VST's FMM spline and not R loess.

Primary sources:
- https://crates.io/api/v1/crates/rcompat-locfit
- https://github.com/deminden/rcompat-locfit/tree/d1fe58bba1cfd8aded7fdd91ab0350b8f65bce5f
- https://stat.ethz.ch/R-manual/R-devel/library/stats/html/splinefun.html
- https://svn.r-project.org/R/tags/R-4-6-1/src/nmath/rnorm.c

The remaining low-df prior also needs actual R seed initialization, sequential
chi-square/normal consumption (sd0 consumes no normal draw), histogram boundary
and density rules, and interpolated degree2 loess. LOWESS/locfit cannot stand
in for that pipeline. Limited official rcompat/author searches find only
lbfgsb/locfit packages, not separate RNG/spline/loess libraries; this is not an
ecosystem-wide absence claim. Qualify concrete consumer behavior before any
dependency or public API decision. Current general-design work is unblocked.
