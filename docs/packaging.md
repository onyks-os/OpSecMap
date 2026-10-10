# Distribution Packages

Besides what `make release-artifacts` builds, OpSecMap publishes distribution packages. Each
package format has its own workflow, because each one builds in its own environment: a distribution
container, Flathub's CI image.

| Workflow | Builds | Installed and run in | Signing identity |
| :-- | :-- | :-- | :-- |
| `package-deb.yml` | `.deb` | Debian 13, Ubuntu 24.04 | `.github/workflows/package-deb.yml@refs/tags/<tag>` |
| `package-rpm.yml` | `.rpm` | Fedora 44 | `.github/workflows/package-rpm.yml@refs/tags/<tag>` |
| `package-arch.yml` | `.pkg.tar.zst` | Arch Linux | `.github/workflows/package-arch.yml@refs/tags/<tag>` |
| `package-flatpak.yml` | `.flatpak` | its own sandbox | `.github/workflows/package-flatpak.yml@refs/tags/<tag>` |

Only the rows whose workflow exists in [`.github/workflows/`](../.github/workflows/) apply to this
repository.

## What every package goes through

1. **Built** by `make package-<format>` — the same target a maintainer runs locally.
2. **Installed in a clean container** of each distribution it targets, through that distribution's
   package manager, and **run** (`make smoke-test-package`). A package that was built but never
   installed is one nobody has tested: a missing dependency, a binary linked against a newer libc,
   a file installed where the program does not look all pass on the build machine.
3. **Described** by a CycloneDX SBOM read from the package itself (`make sbom-files`; see
   [docs/sbom.md](sbom.md)).
4. **Signed** with Sigstore by its own workflow, and attached to the GitHub Release that `release.yml`
   created for the tag. Verify one as [docs/verification.md](verification.md) describes, with the
   identity from the table above.

On a pull request that touches the packaging, the workflow runs steps 1–3 and publishes nothing, so a
broken package is found in review rather than at release time.

## Before tagging

- The version in every packaging file that carries its own copy — `pkgver` in
  `packaging/arch/PKGBUILD` — equals the manifest's. `make release-check` checks it.
- Where a package format is a skeleton (`make todo` lists what is left), its workflow fails on the
  tag: fill it in, or remove the format — its workflow, its `make/features/pkg-*.mk` and its
  `packaging/<format>/` directory.

## Pinned build environments

The Arch build image and the Flatpak CI image are pinned by digest in their workflows, and the
binary-package jobs run on a pinned runner image. Dependabot does not update container digests in
workflows: refresh them deliberately, and note it in the changelog.

For compiled code, the runner's glibc is a floor: the binary runs only on distributions at least as
new. That is why the install test covers the oldest distribution each format supports.
