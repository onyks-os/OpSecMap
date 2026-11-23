# Security assessment — current implementation

This document describes the code in the repository today. Vulnerability reports
belong in [SECURITY.md](../SECURITY.md). Encryption, imports, and a graphical
interface are future designs in [architecture.md](architecture.md), not
implemented controls; the threat model for encryption is at the end of this
page.

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
| Map files read by `open` | JSON text from the path the user typed | Untrusted, size-limited and fully validated |
| Rust library API | Values built by the caller through public constructors | Validated by the library |

The program reads and writes only the map files named in `save` and `open`
commands, and the fictional demo map that `--demo` writes to the system's
temporary directory and deletes. It does not use the network, read environment variables, or contact
any provider.

## Controls present in the code

| Boundary | Current check |
| :--- | :--- |
| Command parsing | Each line becomes a typed command or an error. IDs are validated while parsing; IDs that contain `,` or equal `-` are rejected because the list syntax reserves them. |
| Command execution | Every command goes through the library's public constructors and map methods. A failed command leaves the map unchanged and is reported with its line number. |
| Resource and route construction | Reject blank IDs or labels and empty or duplicate route prerequisites. |
| Dependency map changes | Reject duplicate IDs, references to absent resources, coverage updates for absent resources, and removal of a resource that any route still uses. |
| Scenario construction and evaluation | Reject duplicate or conflicting initial/excluded IDs, then reject scenario IDs absent from the map. |
| Map file reading | Refuse files over 1 MiB before parsing, then check the format name and version, reject unknown fields, and rebuild the map through the public constructors. The map in use is replaced only after every check passes. |
| Map file writing | Write `<path>.tmp` in the same directory, flush it to disk and rename it over the target, so an interrupted save leaves the previous file intact; remove the temporary file on failure. |
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
- **Data at rest.** Map files written by `save` are plain, unencrypted JSON, and
  the program sets no special file permissions: anyone who can read the file
  can read the map. Command files that users redirect into the CLI are plain
  text too. The documentation tells users to keep both fictional until
  encrypted storage exists.
- **Result meaning.** The evaluator reasons only over the routes supplied to it.
  An undocumented or incorrect provider procedure can make a result misleading.
  A resource being accessible does not prove that its data can be restored, that
  an attacker lacks access, or that a backup is intact.

## Supply chain

The runtime dependencies are `serde` and `serde_json`, which read and write the
map file format, and the crates added for encrypted map files: `argon2`,
`chacha20poly1305`, `getrandom`, `rpassword` and `zeroize`. They are declared
but not used by the code yet. `proptest` is used only in tests. Dependency updates, advisories and
licenses are checked as described in [SCA_POLICY.md](../SCA_POLICY.md). There is
no published release artifact yet, so release signatures, reproducible-build
checks and SBOMs cannot currently be used as evidence of this project's security.

## Threat model for encrypted map files (in development)

This section sets the goals for the encrypted format in
[encrypted-format.md](encrypted-format.md). None of it is implemented yet.

| Adversary | Can | Protection |
| :--- | :--- | :--- |
| Someone with a copy of the file: a lost laptop, a backup, a cloud folder | Read and modify the file, try passwords offline | XChaCha20-Poly1305 hides the content and detects any change; Argon2id with 64 MiB per attempt slows password guessing. |
| Someone who replaces the file | Swap in an older or crafted file | A crafted file fails authentication. An older file encrypted with the same password is accepted: the format does not detect rollback. |
| Someone who sends a crafted file | Make `open` read it | The 1 MiB limit and the magic, length and version checks come before key derivation; Argon2 parameters are fixed, not read from the file. |
| Malware or another user with access to the running program | Read memory, the terminal, the environment | **Not protected.** `Zeroizing` shortens the time secrets stay in memory, but copies made by `serde_json` and pages written to swap are not cleared. |

Out of scope: weak passwords (no strength check beyond refusing an empty one),
recovering a forgotten password, hiding the file's size, and keeping the
password secret when it is given through `OPSECMAP_PASSWORD`, which other
processes of the same user can read.
