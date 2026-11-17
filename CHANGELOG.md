<!--
Copyright (c) 2026 onyks-os
SPDX-License-Identifier: AGPL-3.0-or-later
-->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The crate's manifest is currently version `0.1.0`, but the repository has no
tagged or published release yet.

## Unreleased

### Added

- Initial project scaffold generated from
  [TemplateRepository](https://github.com/onyks-os/TemplateRepository).
- Validated resource and route identifiers.
- Resource storage, lookup, insertion and renaming through `DependencyMap`.
- Access and recovery routes with non-empty, unique prerequisite sets.
- Route storage and reference checks in the dependency map.
- In-memory scenarios and fixed-point access evaluation with origin evidence.
- Blocked-route diagnostics with missing prerequisites for unresolved targets.
- Per-resource documentation coverage (`Unknown`, `Incomplete`, `Complete`),
  set through `DependencyMap::set_coverage` and snapshotted in each evaluation.
- A command interpreter on standard input: `resource`, `route`, `coverage`,
  `evaluate`, `remove`, `list`, `help` and `quit`, usable interactively or with a
  file of commands. Failed commands are reported with their line number and
  leave the map unchanged.
- `DependencyMap::remove_route` and `remove_resource`; a resource still used by a
  route is refused with `MapError::ResourceInUse`.
- `--demo` and `--help` options, and an example command file in `tests/fixtures/`.
- A source-only README with badges, example usage, and an AI-use disclosure.
- Unit tests for resource, map and route invariants.
- A project logo in the README.

### Changed

- The minimum supported Rust version is now 1.88, required by current
  development dependencies. Cargo resolves dependency versions compatible with
  it, and CI tests it explicitly.
- Without arguments the CLI now reads commands from standard input; the
  demonstration moved to `--demo`. It evaluates two scenarios and prints each
  resource's status, including documentation coverage, and every blocked route
  with its missing prerequisites. Unknown arguments exit with code `2`.
- Public error types implement `Display` and `std::error::Error`, and the model
  and evaluation types implement `Debug`. `ScenarioError` is now exported.
- `MapError::DuplicateId`, `NotFound` and `DuplicateRouteId` carry the offending
  ID; a blank resource label returns `ResourceError::EmptyLabel`.
- `DependencyMap::empty` is renamed `is_empty`. `AccessEvidence` and
  `BlockedRoute` can no longer be constructed outside the evaluator.
- Public documentation now distinguishes the current example and Rust model from
  planned CLI input, persistence, encryption and graphical presentation.
- Clarified current capabilities, release status, and limitations across the
  README and documentation site.
- Replaced template security, dependency, and release-verification claims with
  descriptions of the current source tree and planned release workflow.
