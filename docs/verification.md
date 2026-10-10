# Release Verification

Every OpSecMap release is published with checksums and a Sigstore signature for every
artifact. This guide shows how to verify an artifact before installing it.

## 1. Verify the Checksums

Download the artifact and `SHA256SUMS` from the
[Releases page](https://github.com/onyks-it/OpSecMap/releases), then:

```bash
sha256sum --check --ignore-missing SHA256SUMS
```

Expected output: `<artifact>: OK`.

## 2. Verify the Sigstore Signature

Release artifacts are signed keylessly through GitHub Actions OIDC — there is no long-lived private
key to steal. Install [`sigstore`](https://pypi.org/project/sigstore/) and verify:

```bash
python -m pip install sigstore

sigstore verify identity \
  --cert-identity "https://github.com/onyks-it/OpSecMap/.github/workflows/release.yml@refs/tags/v0.1.0" \
  --cert-oidc-issuer "https://token.actions.githubusercontent.com" \
  <artifact>
```

## 3. Verifying Signer Identity

The `--cert-identity` value must exactly match the release workflow path and the tag being verified.
A signature that verifies against a *different* identity is not a valid OpSecMap release,
even if the cryptography checks out.

The identity is the workflow that built the artifact. Everything `make release-artifacts` produces is
built and signed by `release.yml`. An artifact built in a workflow of its own — a distribution package
built in that distribution's container — is signed by that workflow, and the document that describes
it names the identity to expect.

## 4. Reproducing the Build

```bash
git clone --branch v0.1.0 https://github.com/onyks-it/OpSecMap.git
cd OpSecMap
make build
sha256sum dist/*
```

<!-- TODO(template): state whether the build is bit-for-bit reproducible, and if not, which parts
     vary (timestamps, paths) and how to normalize them. -->
