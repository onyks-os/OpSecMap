<!--
Copyright (c) 2026 onyks-os
SPDX-License-Identifier: AGPL-3.0-or-later
-->

# OpSecMap — encrypted map file format

**Status:** specified, not implemented yet. Today `save` still writes the
[plain-text format](map-format.md). The reasons for this design are in
[ADR 0002](decisions/0002-encrypt-map-files-with-argon2id-and-xchacha20-poly1305.md).

## Layout, version 1

| Offset | Length | Field |
| :--- | :--- | :--- |
| 0 | 8 | Magic: the ASCII bytes `OPSECMAP` |
| 8 | 1 | Format version: `1` |
| 9 | 16 | Salt, random |
| 25 | 24 | Nonce, random |
| 49 | n + 16 | Ciphertext of the map's JSON, followed by the 16-byte Poly1305 tag |

The first 49 bytes are the **header**. The smallest valid file is 65 bytes.

## Encryption

1. Generate a new random salt and nonce with the operating system's random
   number generator.
2. Derive a 32-byte key from the password's UTF-8 bytes and the salt with
   Argon2id, version 0x13, memory 65,536 KiB, 3 iterations, parallelism 1.
3. Encrypt the [version 1 JSON](map-format.md) with XChaCha20-Poly1305, using
   the key, the nonce, and the whole header as associated data.
4. Write the header followed by the ciphertext and tag.

A salt or nonce is never reused: every save generates both again.

## Decryption

Checks run in this order, so a file that is obviously wrong is refused before
the costly key derivation:

1. The file must start with the magic; otherwise it is not an encrypted map.
   A plain-text map starts with `{` and is read as plain JSON instead.
2. The file must be at least 65 bytes long.
3. The version byte must be `1`.
4. Derive the key from the password and the stored salt, then decrypt with the
   stored nonce and the header as associated data.
5. A tag that does not verify means a wrong password **or** a modified file;
   the two cannot be told apart.
6. The decrypted JSON is then read with every rule of the plain-text format.

The size limit for map files (1 MiB) applies to the encrypted file before any
of these steps.

## Limits

- The file protects the map at rest. It does not protect a map open in a running
  program on a compromised computer.
- There is no password recovery: a forgotten password makes the map unreadable.
  Keep a copy of the file in a second place and the password in a separate one,
  such as a password manager or an offline sheet.
- The file size reveals the approximate size of the map.
