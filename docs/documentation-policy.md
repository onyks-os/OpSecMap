# Documentation policy

This page lists which files must change together, so that the code, its tests and
the documentation describe the same project.

## What to update

| Change | Tests | `CHANGELOG.md` | Also update |
| :--- | :---: | :---: | :--- |
| CLI commands, options, output or exit codes | Yes | Yes | `docs/interfaces.md`, `docs/web/reference/cli.md`, `docs/security-assessment.md`; `README.md` if the quick start changes |
| Public Rust API | Yes | Yes | `docs/interfaces.md`; `docs/architecture.md` if the model changes |
| Evaluation rules or model invariants | Yes | Yes | `docs/architecture.md`, `docs/security-assessment.md` |
| A new input boundary (a file format, an import) | Yes, including property tests | Yes | `docs/security-assessment.md`, `DYNAMIC_ANALYSIS_POLICY.md`, the format's specification |
| Dependencies or minimum Rust version | — | Yes | `DEPENDENCIES.md`; `docs/security-assessment.md` for runtime dependencies |
| Packaging or release workflows | — | Yes | `docs/packaging.md` or `docs/publishing.md` |
| Architectural decision | — | Yes | A new ADR in `docs/decisions/`, plus `docs/architecture.md` |
| Security or governance policies | — | Yes | — |
| Documentation only | — | Only if user-visible | — |

## Before a pull request

1. `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`
   and `cargo test` pass; `make fuzz` too if a parser or validator changed.
2. `CHANGELOG.md` has an entry under `## [Unreleased]` for user-visible changes.
3. The files listed above for the kind of change are updated.
4. Every commit is signed off (`git commit -s`), as described in
   [CONTRIBUTING.md](../CONTRIBUTING.md).

Examples, fixtures and screenshots use fictional data only.
