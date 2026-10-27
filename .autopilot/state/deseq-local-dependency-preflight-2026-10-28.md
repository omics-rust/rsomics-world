# Local-fit dependency qualification

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
