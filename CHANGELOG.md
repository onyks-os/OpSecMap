<!--
Copyright (c) 2026 onyks-it
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
  [TemplateRepository](https://github.com/onyks-it/TemplateRepository).
- Validated resource and route identifiers.
- Resource storage, lookup, insertion and renaming through `DependencyMap`.
- Access and recovery routes with non-empty, unique prerequisite sets.
- Route storage and reference checks in the dependency map.
- In-memory scenarios and fixed-point access evaluation with origin evidence.
- Blocked-route diagnostics with missing prerequisites for unresolved targets.
- A source-only README with badges, example usage, and an AI-use disclosure.
- Unit tests for resource, map and route invariants.
- A project logo in the README.

### Changed

- Public documentation now distinguishes the current example and Rust model from
  planned CLI input, persistence, encryption and graphical presentation.
- Clarified current capabilities, release status, and limitations across the
  README and documentation site.
- Replaced template security, dependency, and release-verification claims with
  descriptions of the current source tree and planned release workflow.
