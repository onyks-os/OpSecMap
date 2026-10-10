# Software Bill of Materials

Every OpSecMap release artifact is published with a CycloneDX SBOM beside it: `<artifact>.cdx.json`.
Each SBOM is checksummed in `SHA256SUMS` and signed with Sigstore like the artifact it describes, so
it can be verified exactly as [docs/verification.md](verification.md) describes.

## One SBOM per artifact, read from the artifact

`scripts/sbom.py` opens each artifact and records what that artifact declares, not what the build
machine had installed:

| Artifact | Requirements come from | Provided by |
| :-- | :-- | :-- |
| wheel, sdist | `Requires-Dist` in its metadata | `pip` |
| npm tarball | `dependencies` in its `package.json` | `npm` |
| `.crate` | `[dependencies]` in its `Cargo.toml` | `cargo` |
| `.deb` / `.rpm` / Arch package | `Depends` / `Requires` / `depend` | the distribution |

For a Rust project, every binary artifact also lists the crates **compiled into it**: the
normal-dependency closure resolved from `Cargo.lock` for the host platform, each with its version,
checksum and licence. Development and build dependencies never reach a user and are not listed.

The described component (`metadata.component`) is the artifact itself, with its SHA-256. The
serial number derives from that digest and the timestamp from `SOURCE_DATE_EPOCH`, so rebuilding the
same artifact produces the same SBOM, byte for byte.

## Reading one

```bash
# What the artifact is
jq '.metadata.component | {name, version, hashes}' <artifact>.cdx.json

# What it requires, and from whom
jq -r '.components[] | [.name, (.properties[]? | select(.name|endswith("provided-by")) | .value)] | @tsv' \
  <artifact>.cdx.json
```

Compare the list against [`DEPENDENCIES.md`](../DEPENDENCIES.md): a component in an SBOM that the
dependency policy does not mention is worth an issue.

## Producing them locally

```bash
make release-artifacts     # builds, writes every SBOM, validates them, checksums everything
make sbom-validate         # re-validate dist/*.cdx.json against the CycloneDX schema
```

Validation is strict: an unknown field fails, not only a missing one.
