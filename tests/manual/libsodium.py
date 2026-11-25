# Copyright (c) 2026 onyks-os
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Checks the encrypted map file format against libsodium, an implementation
independent of the RustCrypto crates that OpSecMap uses.

The script follows only docs/encrypted-format.md. It needs PyNaCl
(``pip install pynacl``), which bundles libsodium.

    python3 tests/manual/libsodium.py vector
        Writes tests/fixtures/encrypted-v1.opsecmap from fixed inputs and
        prints the derived key. The Rust unit tests in src/crypto.rs must
        produce the same bytes.

    OPSECMAP_PASSWORD=... python3 tests/manual/libsodium.py open FILE
        Decrypts FILE, written by `opsecmap save`, and prints its JSON.

All inputs are fictional. Never pass a real password or a real map.
"""

import os
import sys
from pathlib import Path

from nacl import bindings, pwhash

MAGIC = b"OPSECMAP"
VERSION = 1
SALT_LEN = 16
NONCE_LEN = 24
HEADER_LEN = len(MAGIC) + 1 + SALT_LEN + NONCE_LEN
TAG_LEN = 16
# Argon2id, version 0x13, 65,536 KiB, 3 iterations; libsodium always uses
# parallelism 1 and takes the memory limit in bytes.
OPSLIMIT = 3
MEMLIMIT = 65_536 * 1024

# Fixed inputs of the test vector.
PASSWORD = b"correct horse battery staple"
SALT = bytes(range(0x00, 0x10))
NONCE = bytes(range(0x10, 0x28))
PLAINTEXT = (
    b'{\n  "format": "opsecmap-map",\n  "version": 1,\n'
    b'  "resources": [\n    {\n      "id": "phone",\n'
    b'      "label": "Test phone",\n      "coverage": "unknown"\n    }\n'
    b'  ],\n  "routes": []\n}'
)
VECTOR = Path(__file__).resolve().parent.parent / "fixtures" / "encrypted-v1.opsecmap"


def derive_key(password: bytes, salt: bytes) -> bytes:
    return pwhash.argon2id.kdf(32, password, salt, opslimit=OPSLIMIT, memlimit=MEMLIMIT)


def seal(plaintext: bytes, password: bytes, salt: bytes, nonce: bytes) -> bytes:
    header = MAGIC + bytes([VERSION]) + salt + nonce
    key = derive_key(password, salt)
    return header + bindings.crypto_aead_xchacha20poly1305_ietf_encrypt(
        plaintext, header, nonce, key
    )


def open_file(data: bytes, password: bytes) -> bytes:
    if not data.startswith(MAGIC):
        raise SystemExit("not an encrypted map file")
    if len(data) < HEADER_LEN + TAG_LEN:
        raise SystemExit("truncated")
    if data[len(MAGIC)] != VERSION:
        raise SystemExit(f"unsupported version {data[len(MAGIC)]}")
    header = data[:HEADER_LEN]
    salt = header[len(MAGIC) + 1 : len(MAGIC) + 1 + SALT_LEN]
    nonce = header[len(MAGIC) + 1 + SALT_LEN :]
    key = derive_key(password, salt)
    return bindings.crypto_aead_xchacha20poly1305_ietf_decrypt(
        data[HEADER_LEN:], header, nonce, key
    )


def main() -> None:
    if sys.argv[1:] == ["vector"]:
        VECTOR.write_bytes(seal(PLAINTEXT, PASSWORD, SALT, NONCE))
        print("key:", derive_key(PASSWORD, SALT).hex())
        print("wrote", VECTOR)
    elif len(sys.argv) == 3 and sys.argv[1] == "open":
        password = os.environ.get("OPSECMAP_PASSWORD")
        if not password:
            raise SystemExit("set OPSECMAP_PASSWORD to the fictional password")
        sys.stdout.write(open_file(Path(sys.argv[2]).read_bytes(), password.encode()).decode())
    else:
        raise SystemExit(__doc__)


if __name__ == "__main__":
    main()
