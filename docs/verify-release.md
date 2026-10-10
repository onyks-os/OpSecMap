# Verifying Published Releases

[docs/verification.md](verification.md) explains how to check one artifact by hand. This repository
also checks every release as a whole, continuously, with `scripts/verify-release.sh`:

1. every asset has a Sigstore bundle, and every bundle belongs to an asset;
2. every asset was signed by one of this repository's release workflows **at that tag** —
   `release.yml`, or a `package-*.yml` workflow for a distribution package built in its own
   container. A valid signature from any other workflow, fork, or ref is rejected;
3. `SHA256SUMS` matches every file it lists;
4. every hook in `scripts/verify-release.d/` passes. Each publishing module installs one that checks
   its registry serves exactly the bytes attached to the signed release.

The [Verify release](../.github/workflows/verify-release.yml) workflow runs it after every release
workflow succeeds, every week on the latest release, and on demand for any tag. Locally:

```bash
pip install sigstore
make verify-release TAG=v0.1.0
```

A failure here means what a user downloads is not what the release workflow signed — treat it as a
security incident under [SECURITY.md](../SECURITY.md), not as a flaky job.
