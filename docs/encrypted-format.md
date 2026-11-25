<!--
Copyright (c) 2026 onyks-os
SPDX-License-Identifier: AGPL-3.0-or-later
-->

# OpSecMap — encrypted map file format

**Status:** implemented. `save` writes this format, `export` writes the
[plain-text format](map-format.md), and `open` reads both. The library functions
are `encrypt`, `decrypt` and `is_encrypted`. The reasons for this design are in
[ADR 0002](decisions/0002-encrypt-map-files-with-argon2id-and-xchacha20-poly1305.md).
The implementation has not been independently reviewed: until the first
release, prefer fictional maps.

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

## Password

The CLI reads the password from the terminal with echo turned off, not from
standard input, so it works when commands come from a file. `save` asks for it
twice and refuses an empty password or two different ones; `open` asks once,
only after the file has passed the size and header checks. When the environment
variable `OPSECMAP_PASSWORD` is set, its value is used instead of the terminal;
it exists for tests and fictional maps (see [Limits](#limits)).

## Limits

- **A lost password is a lost map.** There is no recovery: nobody, including the
  project, can open the file without the password.
- **A compromised computer is not protected.** The file protects the map at
  rest. Malware or another user who can read the running program's memory, the
  terminal or the keyboard can read the password and the map.
- **Not every copy in memory is cleared.** The password, the key and the
  decrypted JSON are held in `Zeroizing` buffers, which are overwritten when
  dropped. Copies made inside `serde_json` while reading and writing, and the
  map itself, are not; memory written to swap or to a hibernation file can keep
  them on disk.
- **The stack and the CPU registers are not cleared.** While a secret is being
  used, the compiler may put copies of it, or of values computed from it, in
  local variables on the stack and in registers. `Zeroizing` clears only the
  buffer it owns, so those copies stay until the memory is reused. The
  operating system also saves registers when it switches between programs, and
  the terminal driver briefly holds the typed password; neither is under the
  program's control.
- **`OPSECMAP_PASSWORD` is visible** to other processes of the same user, and
  can end up in shell history or scripts. Do not use it for a real password.
- **Weak passwords stay weak.** Argon2id makes each guess cost 64 MiB and a
  fraction of a second, but a short or common password can still be guessed
  offline. Only an empty password is refused; use a long passphrase.
- **Rollback is not detected.** An older file encrypted with the same password
  opens normally.
- **The file size reveals** the approximate size of the map.

## Backups

Because a lost password or a damaged file cannot be recovered:

1. Keep a copy of the encrypted file in a second place, such as another disk or
   a cloud folder. The copy is as safe as the original: it is encrypted.
2. Keep the password **separately** from every copy of the file, for example in
   a password manager or on an offline sheet. A password stored next to the file
   protects nothing.
3. After changing the map, save it and replace the backup copy, then check that
   the copy opens.
4. An `export`ed file is plain text: do not keep one as a backup.
