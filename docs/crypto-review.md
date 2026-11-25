<!--
Copyright (c) 2026 onyks-os
SPDX-License-Identifier: AGPL-3.0-or-later
-->

# OpSecMap — encryption self-review

This checklist reviews the encrypted map files against
[encrypted-format.md](encrypted-format.md) and the
[threat model](security-assessment.md#threat-model-for-encrypted-map-files).
It is the project's own review, **not an independent audit**: it records what
was checked and how, so that an auditor can start from it.

The code under review is `src/crypto.rs` (format and cryptography),
`src/cli/password.rs` (password entry), `src/cli/storage.rs` (files) and the
`save`, `export` and `open` commands in `src/cli/session.rs`.

Each item names its evidence: a test run by `cargo test`, a property run by
`make fuzz`, or a script in `tests/manual/` run by hand.

## Independent implementation

| Check | Evidence |
| :--- | :--- |
| Key derivation gives the same key as libsodium for the same password and salt. | `crypto::tests::key_derivation_matches_libsodium` |
| Encryption with a fixed salt and nonce gives the same file as libsodium, byte for byte. | `crypto::tests::sealed_file_matches_libsodium_byte_for_byte` |
| A file written by libsodium from the specification decrypts and is a valid map. | `crypto::tests::file_written_by_libsodium_decrypts` |
| libsodium decrypts a file written by `opsecmap save` to the same JSON that `export` writes. | `tests/manual/libsodium.py open`, last run 2026-11-25 |

libsodium (through PyNaCl) shares no code with the RustCrypto crates. The test
vector is `tests/fixtures/encrypted-v1.opsecmap`, written by
`python3 tests/manual/libsodium.py vector`; a change to the format, the
parameters or the header makes the three unit tests fail.

## Key derivation

| Check | Evidence |
| :--- | :--- |
| Argon2id, version 0x13, 64 MiB, 3 iterations, parallelism 1, 32-byte key. | Constants in `src/crypto.rs`; libsodium vector |
| The parameters are fixed in the code and never read from the file. | `derive_key` takes only the password and the salt |
| A new 16-byte salt from the operating system for every save. | `encrypt` calls `getrandom::fill`; `crypto::tests::encrypting_twice_gives_a_different_salt_nonce_and_ciphertext` |
| The key is cleared when dropped. | `derive_key` returns `Zeroizing<[u8; 32]>` |

## Encryption

| Check | Evidence |
| :--- | :--- |
| A new 24-byte nonce from the operating system for every save; `seal`, which takes a fixed nonce, is private and called with fixed values only by tests. | `encrypt`; the same test as the salt |
| The whole header (magic, version, salt, nonce) is associated data. | libsodium vector; `crypto::tests::changing_any_byte_of_the_file_is_rejected` |
| The file holds no map text and no password. | `crypto::tests::encrypted_file_does_not_contain_the_plaintext`; `cli::saved_file_is_encrypted_and_holds_no_map_text` |
| `save` never writes plaintext to disk, not even in the temporary file. | `Command::Save` encrypts before calling `write_map_file` |

## Decryption

| Check | Evidence |
| :--- | :--- |
| The size limit, the magic, the length and the version are checked before Argon2. | `decrypt`; `cli::file_over_the_size_limit_is_refused_before_the_password` |
| No input makes `decrypt` panic or succeed without a valid file. | `properties::decrypt_never_panics_or_accepts_arbitrary_bytes` |
| The decrypted JSON is read by `from_json`, which never panics on any text. | `properties::from_json_never_panics_on_any_text`; `properties::from_json_never_panics_on_map_shaped_json` |
| Every file after the magic gets the error of the first check it fails. | `properties::decrypt_classifies_any_file_after_the_magic` |
| Changing any single byte of a valid file is refused. | `crypto::tests::changing_any_byte_of_the_file_is_rejected` (every position) |
| A wrong password and a modified file give the same error. | `CryptoError::WrongPasswordOrDamaged`; `cli::wrong_password_or_changed_byte_leaves_the_previous_map` |
| Plaintext is returned only after the tag verifies, and is cleared when dropped. | The AEAD returns nothing on failure; `decrypt` returns `Zeroizing<Vec<u8>>` |
| A failed `open` leaves the map in use unchanged. | `cli::wrong_password_or_changed_byte_leaves_the_previous_map` |

The tag comparison itself is done by `chacha20poly1305`, which compares in
constant time; that is the library's property and is not tested here.

## Password

| Check | Evidence |
| :--- | :--- |
| Read from the terminal with echo off, never from standard input. | `tests/manual/terminal.py`, last run 2026-11-25 |
| Echo comes back after the password. | `tests/manual/terminal.py` |
| `save` asks twice and refuses a mismatch; `open` asks once. | `tests/manual/terminal.py` |
| An empty password is refused. | `cli::password::tests::the_environment_variable_supplies_the_password`; `cli::empty_password_is_refused_and_writes_nothing` |
| No message contains the password. | `cli::password::tests::messages_do_not_contain_a_password`; `tests/manual/terminal.py` |
| `save` reports an unwritable destination before asking. | `cli::failed_save_is_reported_before_the_password_and_leaves_no_file` |

## Open findings

These are known and accepted for now; each is a candidate for later work.

- **Unicode normalization.** The key is derived from the password's UTF-8 bytes
  as typed. The same password typed as different Unicode sequences, such as an
  accented letter composed or decomposed by another keyboard or system, gives a
  different key and does not open the file.
- **Copies in memory.** `serde_json` buffers, the map's strings, the stack and
  the CPU registers are not cleared (see the
  [format's limits](encrypted-format.md#limits)).
- **Weak passwords.** Only an empty password is refused; there is no strength
  check.
- **Rollback.** An older file encrypted with the same password opens normally.
- **Password comparison.** `save` compares its two readings with `!=`, which is
  not constant-time. Both readings come from the same person at the same
  terminal, so the timing reveals nothing to anyone else.
- **Coverage-guided fuzzing.** `decrypt` and `from_json` are exercised by
  property tests, not yet by a coverage-guided fuzzer such as `cargo fuzz`.

## Running the manual checks

```sh
cargo build
python3 tests/manual/terminal.py           # password entry in a real terminal
python3 tests/manual/libsodium.py vector   # rewrite the test vector (needs PyNaCl)
```

To check a file written by the CLI, save a fictional map with
`OPSECMAP_PASSWORD` set, export the same map, and compare:

```sh
OPSECMAP_PASSWORD='fictional' python3 tests/manual/libsodium.py open map.opsecmap | cmp - map.json
```
