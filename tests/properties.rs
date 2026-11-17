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
