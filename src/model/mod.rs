// Copyright (c) 2026 onyks-os
// SPDX-License-Identifier: AGPL-3.0-or-later
//!
//! Domain model for OpSecMap.

mod evaluation;
mod map;
mod resource;
mod route;
mod scenario;

pub use evaluation::{AccessEvidence, AccessOrigin, BlockedRoute, Evaluation, EvaluationError};
pub use map::{DependencyMap, MapError};
pub use resource::{create_resource, DocumentationCoverage, Resource, ResourceError, ResourceId};
pub use route::{Route, RouteError, RouteId};
pub use scenario::{Scenario, ScenarioError};
