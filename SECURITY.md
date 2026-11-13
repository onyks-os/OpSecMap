# Security policy

## Report a vulnerability

Please report security issues privately so the maintainer can investigate before
details become public. If the repository's
[Security Advisories page](https://github.com/onyks-os/OpSecMap/security/advisories)
offers **Report a vulnerability**, use that form. You can also email
`onyks@protonmail.com`.

Include the affected commit or version, steps to reproduce, the expected and
actual behavior, and the impact you believe the issue has. Please avoid sending
real passwords, recovery codes, or other private account material.

The maintainer will review reports and coordinate disclosure with the reporter.
There is no promised response time, paid bug bounty, or published support window
at this stage.

## Current security scope

OpSecMap is an early Rust library and a CLI that runs a fixed example. The library
validates resource and route structure and evaluates the access described by a map
and scenario in memory. A wrong result, an unexpected panic on valid input, or a
way to bypass documented validation is worth reporting.

The application does not yet load personal maps, persist state, encrypt data, or
provide an interactive interface. It does not store passwords or recovery-code
values. The evaluator cannot verify a provider's real recovery rules, and its
availability result is not a confidentiality or data-integrity assessment. See the
[security assessment](docs/security-assessment.md) for the current boundaries.

There are currently no published GitHub release artifacts to verify or install.
The source and the declared dependencies can be inspected in this repository.
