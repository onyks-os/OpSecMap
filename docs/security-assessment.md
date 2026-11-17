# Security assessment — current implementation

This document describes the code in the repository today. Vulnerability reports
belong in [SECURITY.md](../SECURITY.md). Saving maps, encryption, imports, and a
graphical interface are future designs in [architecture.md](architecture.md), not
implemented controls.

## Assets and boundaries

The model holds resource IDs and labels, documentation coverage, route IDs and
labels, prerequisites, and scenario availability. These values describe
dependencies; they are not passwords, private keys, or recovery-code values. A
map can still reveal sensitive relationships, such as which accounts depend on
one phone and where recovery material is kept.

| Boundary | Input | Trust |
| :--- | :--- | :--- |
| CLI standard input | Command lines typed or redirected from a file | Untrusted text |
| CLI arguments | `--demo`, `--help`, `--version` | Untrusted, matched exactly |
| Rust library API | Values built by the caller through public constructors | Validated by the library |

The program does not write files, read files it opens itself, use the network,
read environment variables, or contact any provider. The map lives in memory and
is lost when the process exits.

## Controls present in the code

| Boundary | Current check |
| :--- | :--- |
| Command parsing | Each line becomes a typed command or an error. IDs are validated while parsing; IDs that contain `,` or equal `-` are rejected because the list syntax reserves them. |
| Command execution | Every command goes through the library's public constructors and map methods. A failed command leaves the map unchanged and is reported with its line number. |
| Resource and route construction | Reject blank IDs or labels and empty or duplicate route prerequisites. |
| Dependency map changes | Reject duplicate IDs, references to absent resources, coverage updates for absent resources, and removal of a resource that any route still uses. |
| Scenario construction and evaluation | Reject duplicate or conflicting initial/excluded IDs, then reject scenario IDs absent from the map. |
| Evaluation | Keep excluded resources inaccessible and derive access only through routes whose prerequisites are accessible. Evaluation always terminates. |

Unit, integration and property tests exercise these rules; the property tests
include arbitrary command lines (see
[DYNAMIC_ANALYSIS_POLICY.md](../DYNAMIC_ANALYSIS_POLICY.md)). The manifest forbids
`unsafe` code, and Clippy denies `unwrap`, `expect` and `panic` outside tests.

## Known limits

- **Resource use.** The CLI reads lines without a length limit, and a map has no
  size limit. A very large input can use a lot of memory or time; with input
  coming from the user's own terminal or file, this is a robustness issue rather
  than a privilege boundary.
- **Data at rest.** Nothing is stored yet. Command files that users write and
  redirect into the CLI are plain text outside the program's control; the
  documentation tells users to keep them fictional until encrypted storage exists.
- **Result meaning.** The evaluator reasons only over the routes supplied to it.
  An undocumented or incorrect provider procedure can make a result misleading.
  A resource being accessible does not prove that its data can be restored, that
  an attacker lacks access, or that a backup is intact.

## Supply chain

The only runtime dependencies are `serde` and `serde_json`, which read and write
the map file format; `proptest` is used only in tests. Dependency updates, advisories and
licenses are checked as described in [SCA_POLICY.md](../SCA_POLICY.md). There is
no published release artifact yet, so release signatures, reproducible-build
checks and SBOMs cannot currently be used as evidence of this project's security.
