// Copyright (c) 2026 onyks-os
// SPDX-License-Identifier: AGPL-3.0-or-later
//!
//! Property-based tests.
//!
//! Every function that parses or validates untrusted input belongs here. The
//! contract: for any input, the function returns a valid result or a documented
//! error — it never panics and never hangs.
//!
//! See `DYNAMIC_ANALYSIS_POLICY.md` for the triage and remediation process.

use proptest::prelude::*;

proptest! {
    #[test]
    fn resource_id_accepts_exactly_nonblank_values(value in ".*") {
        match opsecmap::ResourceId::new(value.clone()) {
            Ok(id) => {
                prop_assert!(!value.trim().is_empty());
                prop_assert_eq!(id.as_str(), value);
            }
            Err(opsecmap::ResourceError::InvalidId) => {
                prop_assert!(value.trim().is_empty());
            }
            Err(_) => prop_assert!(false, "unexpected resource ID error"),
        }
    }

    #[test]
    fn route_id_accepts_exactly_nonblank_values(value in ".*") {
        match opsecmap::RouteId::new(value.clone()) {
            Ok(id) => {
                prop_assert!(!value.trim().is_empty());
                prop_assert_eq!(id.as_str(), value);
            }
            Err(opsecmap::RouteError::InvalidId) => {
                prop_assert!(value.trim().is_empty());
            }
            Err(_) => prop_assert!(false, "unexpected route ID error"),
        }
    }
}

// The CLI command parser is a binary-crate module, which this file cannot
// import: its properties live in `src/cli/command.rs`, module `properties`.

/// The encrypted format's magic, version and header length, as specified in
/// `docs/encrypted-format.md`.
const MAGIC: &[u8] = b"OPSECMAP";
const HEADER_LEN: usize = 49;
const TAG_LEN: usize = 16;

proptest! {
    /// Arbitrary bytes almost never start with the magic: this mostly exercises
    /// the cheap checks, many thousands of times.
    #[test]
    fn decrypt_never_panics_or_accepts_arbitrary_bytes(
        bytes in proptest::collection::vec(any::<u8>(), 0..512),
    ) {
        prop_assert!(opsecmap::decrypt(&bytes, b"test password").is_err());
        prop_assert_eq!(opsecmap::is_encrypted(&bytes), bytes.starts_with(MAGIC));
    }

    /// After the magic, every length and version byte reaches the right check.
    /// One case in 256 has version 1 and a full header, and runs Argon2.
    #[test]
    fn decrypt_classifies_any_file_after_the_magic(
        rest in proptest::collection::vec(any::<u8>(), 0..160),
    ) {
        let file = [MAGIC, &rest].concat();
        let result = opsecmap::decrypt(&file, b"test password");
        if file.len() < HEADER_LEN + TAG_LEN {
            prop_assert!(matches!(result, Err(opsecmap::CryptoError::Truncated)));
        } else if rest[0] != 1 {
            prop_assert!(
                matches!(result, Err(opsecmap::CryptoError::UnsupportedVersion(v)) if v == rest[0])
            );
        } else {
            prop_assert!(matches!(result, Err(opsecmap::CryptoError::WrongPasswordOrDamaged)));
        }
    }
}

proptest! {
    /// The plain-text map reader never panics, on any text or on any valid
    /// JSON shaped like a map file.
    #[test]
    fn from_json_never_panics_on_any_text(text in "\\PC{0,256}") {
        let _ = opsecmap::from_json(&text);
    }

    #[test]
    fn from_json_never_panics_on_map_shaped_json(
        version in any::<u32>(),
        ids in proptest::collection::vec("[a-z,\\- ]{0,6}", 0..6),
        target in "[a-z]{0,4}",
    ) {
        let resources: Vec<String> = ids
            .iter()
            .map(|id| format!(r#"{{"id":{id:?},"label":"Test","coverage":"unknown"}}"#))
            .collect();
        let text = format!(
            r#"{{"format":"opsecmap-map","version":{version},"resources":[{}],"routes":[{{"id":"r","target":{target:?},"prerequisites":{ids:?},"label":"Test"}}]}}"#,
            resources.join(","),
        );
        let _ = opsecmap::from_json(&text);
    }
}

// Changing each byte of a valid file is tested exhaustively, not by sampling,
// in `src/crypto.rs`: every case runs Argon2, so a property here would be slow.
