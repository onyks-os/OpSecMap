# Security policy

## Report a vulnerability

Please report security issues privately so the maintainer can investigate before
details become public. Use the **Report a vulnerability** form on the repository's
[Security Advisories page](https://github.com/onyks-os/OpSecMap/security/advisories),
or email `onyks@protonmail.com`.

Include the affected commit or version, steps to reproduce, the expected and
actual behavior, and the impact you believe the issue has. Reproduce the issue
with fictional resource names: never send real passwords, recovery codes, or a
description of your own accounts.

The maintainer will review reports and coordinate disclosure with the reporter.
There is no promised response time, paid bug bounty, or published support window
at this stage. Reporters who agree are thanked in [HALL_OF_FAME.md](HALL_OF_FAME.md).

## Current security scope

OpSecMap is an early Rust library and a command-line tool. The CLI reads commands
from standard input, interactively or from a file, builds a map in memory, and
evaluates scenarios on it. Nothing is written to disk and nothing is encrypted.

Worth reporting:

- a wrong evaluation result, for example access derived through a route whose
  prerequisites are not all available, or an excluded resource reported as
  accessible;
- a way to bypass documented validation, such as a dangling route reference or a
  removal that breaks one;
- a panic, hang or excessive memory use caused by any command input;
- a command that fails yet changes the map.

The evaluator cannot verify a provider's real recovery rules, and its
availability result is not a confidentiality or data-integrity assessment. See
the [security assessment](docs/security-assessment.md) for the current boundaries.

Saving maps to disk and encrypted storage are planned. Until encrypted storage
exists, the project documents that maps should contain fictional data only.

There are no published release artifacts to verify or install yet.
