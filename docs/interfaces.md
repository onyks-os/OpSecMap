<!--
Copyright (c) 2026 onyks-os
SPDX-License-Identifier: AGPL-3.0-or-later
-->

# OpSecMap — current external interfaces

**Status:** early development. This page describes the interfaces implemented
today; see [the architecture document](architecture.md) for intended behavior.

## Command line

Without arguments, `opsecmap` reads commands from standard input, one per line,
interactively or from a redirected file. Commands add resources and routes, set
documentation coverage, evaluate scenarios, remove elements and list the map as
commands. The map lives in memory; `save <path>` writes it to a file and
`open <path>` replaces it with the map in a file. The
[CLI reference](web/reference/cli.md) documents every command and output line.

`opsecmap --demo` saves a fictional map to a temporary file, reopens it,
deletes the file and evaluates two scenarios on the reopened map. `--help`/`-h` and
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
still uses. `to_json` and `from_json` convert a map to and from the
[map file format](map-format.md) as text; the library itself never opens a
file. See the
public types and documentation in
[`src/lib.rs`](../src/lib.rs) and the model modules.

The API is still evolving before a stable release.

## Files and network

The executable reads and writes only the map files named in `save` and `open`
commands, and the temporary demo file that `--demo` writes to the system's
temporary directory and deletes. They are plain-text JSON, specified in [map-format.md](map-format.md),
and are **not encrypted**. `save` writes `<path>.tmp` beside the target and
renames it into place; `open` refuses files larger than 1 MiB. The executable
does not contact network endpoints. Cargo may access its configured registry when building the
project unless dependencies are already cached or `--offline` is used.
