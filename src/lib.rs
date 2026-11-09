// Copyright (c) 2026 onyks-it
// SPDX-License-Identifier: AGPL-3.0-or-later
//!
//! OpSecMap (OpSecMap)
//!
//! Local-first, zero-trust OpSec dependency mapper. Visualize accounts, devices,
//! backups and recovery paths to find single points of failure before an incident.
//!
//! Everything public in this crate is covered by semantic versioning; items that
//! are not exported here may change in any release.

/// The crate version, taken from `Cargo.toml` at compile time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

mod model;

pub use model::{
    create_resource, AccessEvidence, AccessOrigin, DependencyMap, Evaluation, EvaluationError,
    MapError, Resource, ResourceError, ResourceId, Route, RouteError, RouteId, Scenario,
};

#[cfg(test)]
mod tests {
    use super::VERSION;

    #[test]
    fn version_is_not_empty() {
        assert_ne!(VERSION, "");
    }
}
