# Real signal campaign: first actual qualification failure

Run41224168615/attempt1 at ed0f71c8504ad0a5ed39d30352a9fe89030ffcf2
passes actual native receipt/API admission, complete immutable acquisition,
pinned source installation and three-way QNAME/strand input qualification.
It fails during main/1/source product qualification, before accepted warmups or
paired timing results: tree_bytes first checks a live temporary BAM and then
stats it after deepTools has removed it. The original traceback identifies
evidence/qualification/main-1-source/scratch/_deeptools_cmg_6l3u.bam.
This is a controller sampling race, not a demonstrated native numerical failure
or a storage-budget exceedance. No performance advantage is earned.

Original artifact13051437071 is3784621336B, API SHA256
61d095e8aa9871c7940c2eefd088f30a3fb69258f403580ebb7a3c02e0a5ab3b.
One full external retention request is delegated; full local verification is
still pending and is not claimed by this record. The actual raw GitHub failure
log has been read. This workflow uploads a direct root ZIP/evidence/files.json,
not the peak addon's sole-TAR/artifact-manifest carrier.

The narrow fix samples each entry once with lstat, counts only regular files
and skips only FileNotFoundError for entries removed during the walk. Permission
and other I/O errors still propagate. All resource caps, source/native inputs,
comparison bands, ordering and numerical code remain unchanged.

Existing seven-test controller preparation reproduces the observed ENOENT
with a real temporary pathname removed at metadata access, then passes all7
in0.240s after the fix. Full raw logs remain under
/Volumes/KIOXIA/Developments/tmp/peak-whole-native-integration-20261101.JmnuFX/
as signal-live-tree-red-reproduced.log and signal-live-tree-green-final.log;
earlier preparation attempts remain retained, not rewritten as passing results.
Exact-head controller CI and a new actual campaign remain future gates.
