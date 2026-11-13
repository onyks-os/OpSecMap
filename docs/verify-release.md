# Verifying future releases

No OpSecMap GitHub release or tag is published yet. The
[`verify-release.yml`](../.github/workflows/verify-release.yml) workflow and
[`scripts/verify-release.sh`](../scripts/verify-release.sh) are prepared for a
future release. They are intended to check artifact signatures, workflow
identities, checksums, and registry copies after publication.

There is no current tag or asset on which to run those checks. For the source
checkout available today, use the commands in [verification.md](verification.md).
When the first release exists, this page will record a tested command with its
actual tag and assets.
