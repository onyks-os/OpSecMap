# Security assessment — current implementation

This document describes the code in the repository today. Vulnerability reports
belong in [SECURITY.md](../SECURITY.md). Storage, encryption, imports, and a
graphical interface are future designs in [architecture.md](architecture.md), not
implemented controls.

## Assets and boundaries

The current binary constructs a fixed sample map and scenario in memory. It does
not accept a personal map, read application data files, write application data,
or contact a service. The Rust library accepts maps and scenarios supplied by its
caller. A caller must decide whether its own inputs accurately describe the real
account recovery procedures.

The model holds resource IDs and labels, route IDs and labels, prerequisites, and
scenario availability. These values are descriptions of dependencies. They are
not passwords, private keys, or recovery-code values. A descriptive map can still
reveal sensitive relationships, so future persistence will need a separate
security design before personal maps are stored.

## Controls present in the code

| Boundary | Current check |
| :--- | :--- |
| Resource and route construction | Reject blank IDs or labels and empty or duplicate route prerequisites. |
| Dependency map insertion | Reject duplicate IDs and references to resources absent from the map. |
| Scenario construction and evaluation | Reject duplicate or conflicting initial/excluded IDs, then reject scenario IDs absent from the map. |
| Evaluation | Keep excluded resources inaccessible and derive access only through routes whose prerequisites are accessible. |

Unit and property tests exercise these rules. Run `cargo test` and
`cargo fmt --all -- --check` from the repository root. The manifest forbids
`unsafe` code in this crate.

## Limits of the result

The evaluator reasons only over the routes supplied to it. An undocumented or
incorrect provider procedure can make a result misleading. A resource being
accessible does not prove that its data can be restored, that an attacker lacks
access, or that a backup is intact. The model does not execute recovery actions or
contact providers.

There is no encrypted local storage or published release artifact yet. Release
signatures, reproducible-build claims, and artifact SBOMs cannot currently be
used as evidence of this project's security. Dependency declarations are in
[`Cargo.toml`](../Cargo.toml) and the current inventory is in
[`DEPENDENCIES.md`](../DEPENDENCIES.md).
