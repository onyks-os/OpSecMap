// Copyright (c) 2026 onyks-os
// SPDX-License-Identifier: AGPL-3.0-or-later
//!
//! Encryption of map files with a password.
//!
//! Argon2id derives a key from the password and a random salt; XChaCha20-Poly1305
//! encrypts the map's JSON and authenticates it together with the file header,
//! so a wrong password and a modified file are both rejected. The layout is
//! specified in `docs/encrypted-format.md`. Nothing here reads or writes files.

use std::fmt;

use argon2::{Algorithm, Argon2, Params, Version};
use zeroize::Zeroizing;

const MAGIC: &[u8; 8] = b"OPSECMAP";
const FORMAT_VERSION: u8 = 1;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 24;
const KEY_LEN: usize = 32;
const TAG_LEN: usize = 16;
const SALT_START: usize = MAGIC.len() + 1;
const NONCE_START: usize = SALT_START + SALT_LEN;
const HEADER_LEN: usize = NONCE_START + NONCE_LEN;
const MEMORY_KIB: u32 = 65_536;
const ITERATIONS: u32 = 3;
const PARALLELISM: u32 = 1;

/// An error while encrypting or decrypting a map file.
#[derive(Debug)]
pub enum CryptoError {
    /// The operating system could not provide random bytes for the salt or nonce.
    Random(getrandom::Error),
    /// Argon2 rejected its parameters or the salt.
    KeyDerivation(argon2::Error),
    /// The cipher could not be set up or could not encrypt the map.
    Cipher,
    /// The file does not start with the encrypted format's magic bytes.
    NotEncrypted,
    /// The file is too short to hold a header and an authentication tag.
    Truncated,
    /// The file uses a format version this build cannot read; holds that version.
    UnsupportedVersion(u8),
    /// Authentication failed: the password is wrong, or the file was modified.
    ///
    /// Authenticated encryption cannot tell the two cases apart.
    WrongPasswordOrDamaged,
}

/// Describes the error in one lowercase line, without any secret value.
impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Random(e) => write!(f, "cannot get random bytes from the system: {e}"),
            Self::KeyDerivation(e) => write!(f, "cannot derive a key from the password: {e}"),
            Self::Cipher => f.write_str("cannot encrypt the map"),
            Self::NotEncrypted => f.write_str("not an encrypted OpSecMap map file"),
            Self::Truncated => f.write_str("encrypted map file is truncated"),
            Self::UnsupportedVersion(version) => write!(
                f,
                "unsupported encrypted map file version {version} (this build reads version {FORMAT_VERSION})"
            ),
            Self::WrongPasswordOrDamaged => {
                f.write_str("wrong password, or the map file is damaged or was modified")
            }
        }
    }
}

impl std::error::Error for CryptoError {}

/// Derives the [`KEY_LEN`]-byte encryption key from `password` and `salt` with
/// Argon2id, using the fixed parameters of format version [`FORMAT_VERSION`].
///
/// The key is cleared from memory when the returned value is dropped.
///
/// # Errors
///
/// Returns [`CryptoError::KeyDerivation`] if Argon2 rejects the parameters or a
/// salt shorter than 8 bytes.
fn derive_key(password: &[u8], salt: &[u8]) -> Result<Zeroizing<[u8; KEY_LEN]>, CryptoError> {
    let params = Params::new(MEMORY_KIB, ITERATIONS, PARALLELISM, Some(KEY_LEN))
        .map_err(CryptoError::KeyDerivation)?;
    let a2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = Zeroizing::new([0_u8; KEY_LEN]);
    a2.hash_password_into(password, salt, key.as_mut_slice())
        .map_err(CryptoError::KeyDerivation)?;
    Ok(key)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{derive_key, CryptoError, FORMAT_VERSION, KEY_LEN};

    const SALT: &[u8] = b"fictional-salt-1";

    #[test]
    fn same_password_and_salt_give_the_same_key() {
        let first = derive_key(b"test password", SALT).expect("key must derive");
        let second = derive_key(b"test password", SALT).expect("key must derive");

        assert_eq!(*first, *second);
        assert_ne!(*first, [0_u8; KEY_LEN], "Argon2 must overwrite the zeros");
    }

    #[test]
    fn a_different_salt_or_password_gives_a_different_key() {
        let key = derive_key(b"test password", SALT).expect("key must derive");
        let other_salt =
            derive_key(b"test password", b"fictional-salt-2").expect("key must derive");
        let other_password = derive_key(b"Test password", SALT).expect("key must derive");

        assert_ne!(*key, *other_salt);
        assert_ne!(*key, *other_password);
    }

    #[test]
    fn a_salt_shorter_than_argon2_accepts_is_a_key_derivation_error() {
        assert!(matches!(
            derive_key(b"test password", b"short"),
            Err(CryptoError::KeyDerivation(_))
        ));
    }

    #[test]
    fn unsupported_version_message_names_both_versions() {
        assert_eq!(
            CryptoError::UnsupportedVersion(2).to_string(),
            format!("unsupported encrypted map file version 2 (this build reads version {FORMAT_VERSION})")
        );
    }
}
