# OpSecMap Development Roadmap

This document describes the near-term product direction for OpSecMap. It is not a
release-date commitment; proposed work beyond these milestones is tracked in
[GitHub Issues](https://github.com/onyks-it/OpSecMap/issues).

## Current capabilities

- Validated resource and route identifiers.
- Resource creation, insertion, lookup and renaming.
- Route construction with a target and a non-empty set of unique prerequisites.
- Automated checks for the implemented domain invariants.

The command-line interface currently demonstrates the domain model. OpSecMap is
not yet ready for storing or evaluating real operational maps.

## Dependency graph

**Goal:** build a complete, internally consistent in-memory representation of an
OpSec dependency map.

| Item | Description |
| :--- | :---------- |
| Route storage | Store and query routes alongside resources. |
| Referential integrity | Reject routes whose target or prerequisites are absent. |
| Mutation rules | Prevent operations from leaving dangling references. |
| Import validation | Apply the same invariants to data loaded from external representations. |

## Scenario evaluation

**Goal:** explain which resources remain accessible under a declared loss or
failure scenario.

| Item | Description |
| :--- | :---------- |
| Scenario model | Represent initially available and explicitly excluded resources. |
| Fixed-point evaluator | Resolve conjunctive prerequisites and alternative routes deterministically. |
| Evidence | Report the routes supporting each derived result and unresolved prerequisites. |
| CLI | Accept a map and scenario, then render structured diagnostics and results. |

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
