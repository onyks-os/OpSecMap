<!--
Copyright (c) 2026 onyks-it
SPDX-License-Identifier: AGPL-3.0-or-later
-->

# OpSecMap — current external interfaces

**Status:** early development. This page describes the interfaces implemented
today; see [the architecture document](architecture.md) for intended behavior.

## Command line

Run the fixed demonstration with `opsecmap` (or `cargo run --` from the repository).
It constructs four sample resources and two routes to an email account, then
evaluates a scenario where the phone is initially accessible and the email
account is excluded. The command prints the routes and resulting access.

`opsecmap --version` and `opsecmap -V` print `opsecmap 0.1.0` and exit.
No other argument is currently parsed; unsupported arguments are ignored and the
demonstration still runs. There is no `--help` handler, configuration file, map
input, scenario input, or output format flag.

| Exit code | Meaning |
| :-------- | :------ |
| `0` | Version printed or demonstration completed. |
| `1` | The built-in example could not be constructed or evaluated. |

## Rust library

The `opsecmap` crate exposes resources, routes, a dependency map, scenarios and
evaluations. Callers construct a map and scenario in memory, then call
`Evaluation::evaluate(&map, &scenario)`. The returned evaluation can answer
whether a resource is accessible and give the initial or route-based evidence
for an accessible resource. It also exposes blocked routes and their missing
prerequisites for targets that remain inaccessible and are not excluded. See the
public types and documentation in
[`src/lib.rs`](../src/lib.rs) and the model modules.

The API is still evolving before a stable release.

## Files and network

The executable does not read or write application data files and does not contact
network endpoints. Cargo may access its configured registry when building the
project unless dependencies are already cached or `--offline` is used.
