<!--
Copyright (c) 2026 onyks-it
SPDX-License-Identifier: AGPL-3.0-or-later
-->

# OpSecMap — current external interfaces

**Status:** early development. This page describes the interfaces implemented
today; see [the architecture document](architecture.md) for intended behavior.

## Command line

Without arguments, `opsecmap` reads commands from standard input, one per line,
interactively or from a redirected file. Commands add resources and routes, set
documentation coverage, evaluate scenarios, remove elements and list the map as
commands. The map lives in memory only. The
[CLI reference](web/reference/cli.md) documents every command and output line.

`opsecmap --demo` evaluates two scenarios on a fictional map. `--help`/`-h` and
`--version`/`-V` print usage or the version. There is no configuration file or
output format flag.

| Exit code | Meaning |
| :-------- | :------ |
| `0` | Every command succeeded, or a flag completed. |
| `1` | A command failed, input could not be read, or the demo failed. |
| `2` | Unknown argument. |

## Rust library

The `opsecmap` crate exposes resources, routes, a dependency map, scenarios and
evaluations. Callers construct a map and scenario in memory, then call
`Evaluation::evaluate(&map, &scenario)`. The returned evaluation can answer
whether a resource is accessible and give the initial or route-based evidence
for an accessible resource. It also exposes blocked routes and their missing
prerequisites for targets that remain inaccessible and are not excluded, and
the documentation coverage each resource had when the evaluation ran. The map
can remove routes and resources, refusing to remove a resource that a route
still uses. See the
public types and documentation in
[`src/lib.rs`](../src/lib.rs) and the model modules.

The API is still evolving before a stable release.

## Files and network

The executable does not read or write application data files and does not contact
network endpoints. Cargo may access its configured registry when building the
project unless dependencies are already cached or `--offline` is used.
