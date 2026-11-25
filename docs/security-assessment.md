# Security assessment — current implementation

This document describes the code in the repository today. Vulnerability reports
belong in [SECURITY.md](../SECURITY.md). Imports and a graphical interface are
future designs in [architecture.md](architecture.md), not implemented controls.
The threat model for encrypted map files is at the end of this page.

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
| Map files read by `open` | Encrypted or plain JSON bytes from the path the user typed | Untrusted, size-limited, authenticated if encrypted, fully validated |
| Password | The terminal with echo off, or `OPSECMAP_PASSWORD` | Secret; never printed or written |
| Rust library API | Values built by the caller through public constructors | Validated by the library |

The program reads and writes only the map files named in `save`, `export` and
`open` commands, and the fictional demo map that `--demo` exports to the
system's temporary directory and deletes. It reads only one environment
variable, `OPSECMAP_PASSWORD`, does not use the network, and does not contact
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
| Map file writing | Write `<path>.tmp` in the same directory, flush it to disk and rename it over the target, so an interrupted save leaves the previous file intact; remove the temporary file on failure. `save` encrypts before writing, so its temporary file is never plain text, and checks that the destination is writable before asking for the password. |
| Encryption | `save` derives a key with Argon2id (64 MiB, 3 iterations) from the password and a new random salt, and encrypts with XChaCha20-Poly1305 and a new random nonce; the header is authenticated as associated data. |
| Encrypted file reading | After the 1 MiB limit, check the magic, the length and the version before key derivation; refuse the file unless authentication succeeds, then validate the JSON like a plain file. A wrong password and a modified file give the same error. |
| Password handling | Read from the terminal with echo off, never from standard input; refuse an empty password; `save` asks twice and refuses a mismatch. The password, the key and decrypted JSON are held in `Zeroizing` buffers. |
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
- **Data at rest.** Files written by `save` are encrypted; files written by
  `export`, and command files that users redirect into the CLI, are plain text.
  The program sets no special file permissions. The encryption has not been
  independently reviewed, and the documentation recommends fictional maps until
  the first release. Its limits are in the threat model below.
- **Result meaning.** The evaluator reasons only over the routes supplied to it.
  An undocumented or incorrect provider procedure can make a result misleading.
  A resource being accessible does not prove that its data can be restored, that
  an attacker lacks access, or that a backup is intact.

## Supply chain

The runtime dependencies are `serde` and `serde_json`, which read and write the
map file format, and the crates added for encrypted map files: `argon2`,
`chacha20poly1305`, `getrandom`, `rpassword` and `zeroize`. `proptest` is used
only in tests. Dependency updates, advisories and
licenses are checked as described in [SCA_POLICY.md](../SCA_POLICY.md). There is
no published release artifact yet, so release signatures, reproducible-build
checks and SBOMs cannot currently be used as evidence of this project's security.

## Threat model for encrypted map files

This section covers the encrypted format in
[encrypted-format.md](encrypted-format.md), as implemented. Tests check a wrong
password, a changed byte in each field, truncated and oversized files, an
unsupported version, and that a saved file contains no map text.

| Adversary | Can | Protection |
| :--- | :--- | :--- |
| Someone with a copy of the file: a lost laptop, a backup, a cloud folder | Read and modify the file, try passwords offline | XChaCha20-Poly1305 hides the content and detects any change; Argon2id with 64 MiB per attempt slows password guessing. |
| Someone who replaces the file | Swap in an older or crafted file | A crafted file fails authentication. An older file encrypted with the same password is accepted: the format does not detect rollback. |
| Someone who sends a crafted file | Make `open` read it | The 1 MiB limit and the magic, length and version checks come before key derivation; Argon2 parameters are fixed, not read from the file. |
| Malware or another user with access to the running program | Read memory, the terminal, the environment | **Not protected.** `Zeroizing` shortens the time secrets stay in memory, but copies made by `serde_json`, copies on the stack and in CPU registers, and pages written to swap are not cleared. |

Out of scope: weak passwords (no strength check beyond refusing an empty one),
recovering a forgotten password, hiding the file's size, and keeping the
password secret when it is given through `OPSECMAP_PASSWORD`, which other
processes of the same user can read.

**A lost password is a lost map**: there is no recovery mechanism. The
[backup advice](encrypted-format.md#backups) is to keep a copy of the encrypted
file in a second place and the password separately from every copy.
