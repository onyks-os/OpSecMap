# Dependency review

The repository configures Dependabot updates for Cargo, GitHub Actions, and the
documentation toolchain in [`.github/dependabot.yml`](.github/dependabot.yml).
The CI workflow also configures a Rust dependency audit. Direct dependencies
and their purpose are listed in [DEPENDENCIES.md](DEPENDENCIES.md).

Before accepting an update, review its changelog, license, transitive changes,
and relevance to this project. Run the applicable build and tests. A security
alert should be investigated in the context of how OpSecMap uses that
dependency; accepted risks should be recorded rather than silently ignored.

There is no published release or committed remediation-time guarantee at this
stage. Security reports follow [SECURITY.md](SECURITY.md).
