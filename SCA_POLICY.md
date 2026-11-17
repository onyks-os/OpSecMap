# Dependency review

Direct dependencies and their purpose are listed in [DEPENDENCIES.md](DEPENDENCIES.md).
At runtime the library and the CLI depend only on `serde` and `serde_json`, used
for the map file format.

Automated checks:

- [Dependabot](.github/dependabot.yml) proposes weekly updates for Cargo, GitHub
  Actions, and the documentation toolchain.
- The CI workflow audits `Cargo.lock` against the RustSec advisory database.
- The [dependency review](.github/workflows/dependency-review.yml) workflow blocks
  pull requests that add a dependency with a known high-severity vulnerability, or
  under a license incompatible with AGPL-3.0-or-later distribution (SSPL-1.0,
  BUSL-1.1).
- `.cargo/config.toml` makes Cargo prefer dependency versions compatible with the
  minimum Rust version declared in `Cargo.toml`, and CI tests that version.

Before adding or accepting a dependency, review its maintenance, license,
transitive dependencies and relevance to this project, then run the build and
tests. Planned features will add dependencies for serialization and for
encryption: cryptographic code must come from maintained, widely reviewed crates,
never from new or little-used ones. A security alert is investigated in the
context of how OpSecMap uses that dependency; accepted risks are recorded rather
than silently ignored.

There is no published release or committed remediation-time guarantee at this
stage. Security reports follow [SECURITY.md](SECURITY.md).
