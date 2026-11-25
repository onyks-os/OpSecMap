<!--
Copyright (c) 2026 onyks-os
SPDX-License-Identifier: AGPL-3.0-or-later
-->

# ADR 0002: Encrypt map files with Argon2id and XChaCha20-Poly1305

## Status

Accepted during 0.1.0 development

## Context

`save` writes a dependency map as plain JSON. A real map reveals which accounts
depend on one phone and where recovery material is kept, so anyone with a copy
of the file learns how to attack or lock out its owner. Until map files are
encrypted the project tells users to store fictional data only.

The requirements, from [architecture.md](../architecture.md) section 7:

* a password, not a key file, protects the map;
* a modified file is detected and rejected, not silently loaded;
* failed decryption leaves the map in use unchanged;
* cryptography comes from maintained, widely reviewed libraries.

## Decision

We decided to encrypt map files with a format of our own that combines two
standard primitives, both from the RustCrypto project:

* **Argon2id** (`argon2` crate) derives a 32-byte key from the password and a
  random 16-byte salt, with fixed parameters: 64 MiB of memory, 3 passes, 1 lane.
* **XChaCha20-Poly1305** (`chacha20poly1305` crate) encrypts the map's JSON with
  a random 24-byte nonce. The file header (magic, version, salt, nonce) is
  passed as associated data, so it is authenticated too.

Salt and nonce come from `getrandom` and are new on every save. The parameters
are fixed per format version instead of being stored in the file: a file cannot
ask for an expensive derivation, and changing them means a new version.
Passwords, keys and decrypted maps are held in `zeroize::Zeroizing` so they are
cleared when dropped. The CLI reads the password from the terminal with
`rpassword`; `OPSECMAP_PASSWORD` overrides it for tests and automation.

The library does the encryption, without file or terminal I/O; the CLI reads
the password and the file. `save` encrypts; plain JSON is written only by an
explicit `export` command, and `open` accepts both. The format is specified in
[encrypted-format.md](../encrypted-format.md).

### Alternatives Considered

| Alternative | Why it was rejected |
| :---------- | :------------------ |
| The `age` format and crate | A reviewed format that other tools can decrypt, but the crate brings a much larger dependency tree, and its passphrase mode uses scrypt rather than Argon2id. |
| AES-256-GCM | Equally standard, but its 12-byte nonce makes random nonces riskier, and it is slow without hardware AES. XChaCha20's 24-byte nonce is safe to pick at random. |
| Parameters stored in the header | Flexible, but a crafted file could demand gigabytes of memory; every bound would need checking before derivation. |
| Password as a command argument | It would end up in shell history and command files. |

## Consequences

* **Pros**:
  * A copied file reveals nothing but its size, and any change to it is detected.
  * Two saves of the same map with the same password produce different files.
  * All cryptographic code comes from maintained crates with published reviews.
* **Cons**:
  * The format is specific to OpSecMap: no other tool can decrypt it.
  * A lost password cannot be recovered; the map is lost with it.
  * Five new runtime dependencies, about 30 crates in total.
* **Neutral / follow-up work**:
  * Encryption does not protect a map open on a compromised computer, nor data
    copied to swap or left in memory by `serde_json`.
  * Changing Argon2 parameters later requires format version 2 and reading both.

## References

* [encrypted-format.md](../encrypted-format.md)
* [RFC 9106: Argon2](https://www.rfc-editor.org/rfc/rfc9106)
* [XChaCha20-Poly1305 draft](https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-xchacha)
