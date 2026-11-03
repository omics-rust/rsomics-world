# Count0.2 delivery frontier

After projection commit2b5e450 exact-head native-four CI41607175618 passes,
prepare a genuine0.2 minor release of the existing count/normalize workflow
plus original-CIGAR corrections and extension5/3/read2pos. Do not add another
operation merely to postpone delivery. Unimplemented junction/read-group/
long-read/assignment and larger overlap-policy slices remain explicitly out.

The public CountOptions projection field is an API addition. Bump package
and root lock entry to0.2.0; preserve the complete external dependency graph.
`scripts/delivery.py source()` verifies licenses/provenance.json lock_sha256
against the exact Cargo.lock, so update that binding for the root-version-only
change after verifying normal/build dependency graphs remain unchanged.
Do not regenerate or relabel old third-party license bodies or old releases.

Existing delivery helper smoke runs nine installed commands: version/help,
two-library count, three normalizations, stdin and two protected failures.
For0.2 add one real projected-count installed call using owned projection
fixtures, checking its complete matrix/summary and common JSON result. Update
the explicit call total. Source/debug/release oracle tests already cover
projection; the additional installed call establishes the delivered CLI.

Run one fresh whole-delivery review, native-four source/package/install gate,
and independently adopt exact-head original artifacts before publication.
Keep LinuxARM coverage frozen, not live. Reuse existing proven carrier and
public-install machinery where applicable; no parallel publication or token
mutation. New measured production source is bound to2b5e450 and the local
preserved native binary, not a claim that every CI binary was benchmarked.

Publish new GitHub0.2 assets/tag only after qualification, then fresh public
download and installed lifecycle. Preserve0.1.0 immutable tag/assets and
registry archive. Registry auth has a separately documented403 blocker;
do not retry unchanged publication credentials or claim crates.io0.2 from a
successful GitHub delivery. The all30 portfolio goal remains active.
