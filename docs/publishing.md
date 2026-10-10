# Publishing to crates.io

[`publish-crates.yml`](../.github/workflows/publish-crates.yml) runs on every `v*` tag that is not a
pre-release. `cargo publish` cannot upload a prebuilt `.crate`: it packages the tree again. So the
workflow first downloads the `.crate` from the GitHub Release, verifies its Sigstore signature,
packages the tag itself, and **refuses to publish unless the two are identical byte for byte**. Only
then does it run `cargo publish`.

## One-time set-up

1. **First publish.** crates.io does not allow a crate's first version through Trusted Publishing.
   Create an API token scoped to publish-new, store it as the `CARGO_REGISTRY_TOKEN` secret of an
   environment named `crates-io` (*Settings → Environments*), and push the first tag.
2. **Trusted Publishing.** In the crate's settings on crates.io, add a trusted publisher: repository
   `onyks-it/OpSecMap`, workflow `publish-crates.yml`, environment `crates-io`. Set the
   repository variable `CRATES_IO_TRUSTED_PUBLISHING` to `true`, then delete the secret and revoke
   the token.

## Checked afterwards

The `crates-io.sh` hook of [`scripts/verify-release.sh`](../scripts/verify-release.sh) confirms that
crates.io's checksum for the version is the SHA-256 of the signed `.crate`.
