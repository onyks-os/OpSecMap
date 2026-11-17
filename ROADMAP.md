# OpSecMap Development Roadmap

This document describes the near-term product direction for OpSecMap. It is not a
release-date commitment; proposed work beyond these milestones is tracked in
[GitHub Issues](https://github.com/onyks-os/OpSecMap/issues).

## Current capabilities

- Validated resource and route identifiers.
- Resource creation, insertion, lookup and renaming.
- Route construction with a target and a non-empty set of unique prerequisites.
- Route storage and reference checks against the resources in the map.
- In-memory scenarios with initially available and excluded resources.
- Evaluation of AND prerequisites, alternative routes, chains and cycles, with
  initial or route-based evidence for accessible resources.
- Blocked-route records with the missing prerequisites for unresolved targets.
- Per-resource documentation coverage, kept separate from accessibility.
- Removal of routes and resources without leaving dangling references.
- A command-line interface that builds a map from commands on standard input,
  evaluates scenarios and lists the map as reusable commands.
- Automated checks for the implemented domain invariants.

The map lives in memory only: there is no persistence or encryption yet.

## Dependency graph

**Goal:** build a complete, internally consistent in-memory representation of an
OpSec dependency map.

| Item | Description |
| :--- | :---------- |
| Import validation | Apply the same invariants to data loaded from external representations. |

## Later milestones

- Versioned local persistence and encrypted state.
- Interactive visualization over the same domain model and evaluator.
- Packaging and release workflows for supported platforms.

## Explicitly out of scope

- Storing passwords, private keys or recovery-code values.
- Automatically executing or verifying provider recovery procedures.
- Treating account accessibility as proof of confidentiality or data integrity.
- Assigning an opaque global security score.

## How to influence the roadmap

Open an issue describing the use case you are blocked on. Roadmap items are
prioritized by user impact and alignment with the documented architecture, not by
request order.
