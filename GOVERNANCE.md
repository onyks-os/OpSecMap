# Governance

This document describes how decisions are made in OpSecMap.

## Model

OpSecMap has a single maintainer, listed in [MAINTAINERS.md](MAINTAINERS.md), who has
final authority over the project. If regular contributors join, this document will be
revised to delegate areas of responsibility to them.

## Decision-making

| Decision type | Process |
| :--- | :--- |
| Bug fix, documentation, refactor | A pull request or a direct commit by the maintainer; CI must pass. |
| New feature | An issue describing the problem first; then a pull request. |
| Architectural or breaking change | An [ADR](docs/decisions/) recording the decision and its trade-offs. |
| Change to how map data is stored, encrypted or parsed | Explicit review against [docs/security-assessment.md](docs/security-assessment.md), updated in the same change. |
| License or code of conduct change | Announced in an issue at least 14 days in advance. |

Contributions from others are reviewed by the maintainer before they are merged.
Disagreements are discussed in the issue or pull request; if no agreement is reached,
the maintainer decides and records the rationale there, or in an ADR for
architectural matters.

## Proposing a change

1. Open an issue describing the problem, not only the solution.
2. For an architectural change, propose an ADR using
   [`docs/decisions/0000-adr-template.md`](docs/decisions/0000-adr-template.md).
3. Once the approach is agreed, submit the implementation as a pull request.

## Communication channels

- **GitHub Issues**: bug reports, feature requests and design discussions.
- **GitHub Security Advisories**: private vulnerability reports, see [SECURITY.md](SECURITY.md).
- **Email**: `onyks@protonmail.com`, for anything that cannot be public.

Project decisions are made in public so that they remain auditable. Private discussion
is limited to security reports and code of conduct matters.

## Changing this document

Changes to this document are made by pull request and recorded in the changelog.
