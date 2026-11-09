// Copyright (c) 2026 onyks-it
// SPDX-License-Identifier: AGPL-3.0-or-later
//!
//! Domain model for OpSecMap.

mod evalutation;
mod map;
mod resource;
mod route;
mod scenario;

pub use evalutation::{AccessEvidence, AccessOrigin, Evaluation, EvaluationError};
pub use map::{DependencyMap, MapError};
pub use resource::{create_resource, Resource, ResourceError, ResourceId};
pub use route::{Route, RouteError, RouteId};
pub use scenario::Scenario;
