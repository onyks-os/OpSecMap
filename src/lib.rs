// Copyright (c) 2026 onyks-it
// SPDX-License-Identifier: AGPL-3.0-or-later
//!
//! OpSecMap
//!
//! In-memory model and evaluator for account access dependencies. The library
//! records reachable resources, their access origins, and missing prerequisites
//! for blocked routes. The binary currently runs a fixed example.
//!
//! The public API is still evolving during early development.

/// The crate version, taken from `Cargo.toml` at compile time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

mod model;

pub use model::{
    create_resource, AccessEvidence, AccessOrigin, BlockedRoute, DependencyMap, Evaluation,
    EvaluationError, MapError, Resource, ResourceError, ResourceId, Route, RouteError, RouteId,
    Scenario,
};

#[cfg(test)]
mod tests {
    use super::VERSION;

    #[test]
    fn version_is_not_empty() {
        assert_ne!(VERSION, "");
    }
}
