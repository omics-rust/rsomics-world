# Registry authentication scope check

Read-only revalidation, not another publish attempt. No secret value is
retrieved, printed, stored or changed; no repository access is broadened.

Current organization secret CARGO_REGISTRY_TOKEN has selected visibility,
updated_at2026-09-08T20:26:08Z and18 selected repositories. Actual selected
targets include rsomics-bed/1316819000 and rsomics-pileup/1251135163. Both
repositories report zero repository-level secrets, excluding a same-name local
secret overriding the organization value. Pileup is a public repository.

Latest actual pileup Publish invocation40583646680 at
1bc989e25a42ce6c0c76e3572b41d4da283e74f2, created2026-10-27T21:04:34Z,
is failure. Root reads its exact workflow: manual main-only publication,
no environment-level override, CARGO_REGISTRY_TOKEN from secrets, external
hosted temporary Cargo/Rustup/target paths. The failed-run log has one token
environment field, nonempty and fully masked. Root examines only the presence
booleans and sanitized diagnostic, never a token. Actual server result:
`status 403 Forbidden: authentication failed`.

This is not missing selected-repository access, an empty injected variable,
repository secret shadowing, a sudo or local storage problem. The effective
credential was rejected; expiry/revocation/other account cause is not inferred
from this generic message. Organization metadata remains unchanged since that
attempt, so no blind authenticated registry retry is authorized by this audit.
An updated valid organization secret or another demonstrated credential-state
change is needed before the next qualified publication attempt. Update it in
GitHub, not by posting a token in chat. Other selected repository membership
must still be checked individually when a product becomes release-ready.

Fresh public sparse-index reads on this audit observe BED versions[0.1.0]
only and pileup latest0.9.0 without0.9.1. BED GitHub0.6.0 is therefore a
separate delivery, not a registry publication. The missing pileup producer
registry version continues to block normalized BAM/RNA source packaging;
do not hide the dependency behind a Git source or weaken package validation.
This audit does not erase any original failure or qualify another product.
