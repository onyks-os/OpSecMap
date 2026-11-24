// Copyright (c) 2026 onyks-os
// SPDX-License-Identifier: AGPL-3.0-or-later
//!
//! OpSecMap
//!
//! In-memory model and evaluator for account access dependencies. The library
//! records reachable resources, their access origins, and missing prerequisites
//! for blocked routes. The binary reads map commands from standard input.
//!
//! The public API is still evolving during early development.

/// The crate version, taken from `Cargo.toml` at compile time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

mod crypto;
mod model;
mod persistence;

pub use crypto::CryptoError;
pub use model::{
    create_resource, AccessEvidence, AccessOrigin, BlockedRoute, DependencyMap,
    DocumentationCoverage, Evaluation, EvaluationError, MapError, Resource, ResourceError,
    ResourceId, Route, RouteError, RouteId, Scenario, ScenarioError,
};
pub use persistence::{from_json, to_json, MapFileError};

#[cfg(test)]
mod tests {
    use super::VERSION;

    #[test]
    fn version_is_not_empty() {
        assert_ne!(VERSION, "");
    }
}
