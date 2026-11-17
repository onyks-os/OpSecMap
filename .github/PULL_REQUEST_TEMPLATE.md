## Summary

<!-- What does this change do, and why? Link the issue it closes: Closes #123 -->

## Type of change

- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that changes existing behavior)
- [ ] Documentation only
- [ ] CI / build / tooling

## How has this been tested?

<!-- Describe the tests you ran and how a reviewer can reproduce them. -->

## Checklist

- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings` and `cargo test` pass locally (`make verify` also runs `cargo audit`).
- [ ] Tests are added or updated for the change.
- [ ] `CHANGELOG.md` is updated under `## [Unreleased]`, if the change is visible to users.
- [ ] Documentation required by [`docs/documentation-policy.md`](../docs/documentation-policy.md) is updated.
- [ ] An [ADR](../docs/decisions/) is added if this change is architectural.
- [ ] All commits are signed off (`git commit -s`) per the [DCO](../CONTRIBUTING.md#commits-and-pull-requests).

## Security considerations

<!-- Does this change touch an input boundary (CLI commands, map files), stored or encrypted data,
     or add a dependency? If yes, say how docs/security-assessment.md is affected.
     If no, write "None". Never paste real account or recovery details. -->
