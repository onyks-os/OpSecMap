# Dependency review

Direct dependencies and their purpose are listed in [DEPENDENCIES.md](DEPENDENCIES.md).
At runtime the library and the CLI depend on `serde` and `serde_json` for the
map file format, and on `argon2`, `chacha20poly1305`, `getrandom`, `rpassword`
and `zeroize` for encrypted map files. The cryptographic crates all come from
the RustCrypto project; the choice is recorded in
[ADR 0002](docs/decisions/0002-encrypt-map-files-with-argon2id-and-xchacha20-poly1305.md).

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
tests. Cryptographic code must come from maintained, widely reviewed crates,
never from new or little-used ones, and OpSecMap combines their primitives
without implementing any of its own. A security alert is investigated in the
context of how OpSecMap uses that dependency; accepted risks are recorded rather
than silently ignored.

There is no published release or committed remediation-time guarantee at this
stage. Security reports follow [SECURITY.md](SECURITY.md).
