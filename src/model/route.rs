// Copyright (c) 2026 onyks-it
// SPDX-License-Identifier: AGPL-3.0-or-later
//!
//! Domain model for OpSecMap.

use super::resource::ResourceId;
use std::collections::HashSet;

/// Stable identifier for an access or recovery route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteId(String);

impl RouteId {
    /// Creates a validated route identifier.
    ///
    /// # Errors
    ///
    /// Returns [`RouteError::InvalidId`] when `value` is empty or contains only
    /// whitespace.
    pub fn new(value: String) -> Result<Self, RouteError> {
        if value.trim().is_empty() {
            return Err(RouteError::InvalidId);
        }
        Ok(Self(value))
    }

    /// Returns the identifier as borrowed text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A named procedure that makes one target resource accessible.
///
/// Every prerequisite belongs to the same route and will therefore be required
/// together. Separate routes with the same target represent alternatives.
pub struct Route {
    /// Stable identity of this procedure.
    id: RouteId,
    /// Human-readable description of the procedure.
    label: String,
    /// Resource made accessible when the procedure succeeds.
    target: ResourceId,
    /// Resources that must all be accessible for this procedure to succeed.
    prerequisites: Vec<ResourceId>,
}

impl Route {
    /// Creates a route from its identity, target, and prerequisite group.
    ///
    /// # Errors
    ///
    /// Returns [`RouteError::EmptyLabel`] for a blank label,
    /// [`RouteError::EmptyPrerequisites`] when the route has no prerequisites,
    /// or [`RouteError::DuplicatePrerequisites`] when the same resource appears
    /// more than once in the prerequisite group.
    pub fn new(
        id: RouteId,
        label: String,
        target: ResourceId,
        prerequisites: Vec<ResourceId>,
    ) -> Result<Self, RouteError> {
        if label.trim().is_empty() {
            return Err(RouteError::EmptyLabel);
        }
        if prerequisites.is_empty() {
            return Err(RouteError::EmptyPrerequisites);
        }
        let mut seen = HashSet::new();
        if prerequisites
            .iter()
            .any(|prerequisite| !seen.insert(prerequisite))
        {
            return Err(RouteError::DuplicatePrerequisites);
        }
        Ok(Self {
            id,
            label,
            target,
            prerequisites,
        })
    }

    /// Returns the route's stable identifier.
    #[must_use]
    pub fn id(&self) -> &RouteId {
        &self.id
    }

    /// Returns the human-readable route label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the resource made accessible by this route.
    #[must_use]
    pub fn target(&self) -> &ResourceId {
        &self.target
    }

    /// Returns all prerequisites as a read-only slice.
    #[must_use]
    pub fn prerequisites(&self) -> &[ResourceId] {
        &self.prerequisites
    }
}

/// Errors produced while constructing route values.
#[derive(Debug, PartialEq, Eq)]
pub enum RouteError {
    /// The route identifier is empty or contains only whitespace.
    InvalidId,
    /// The human-readable label is empty or contains only whitespace.
    EmptyLabel,
    /// The route does not declare any prerequisite resource.
    EmptyPrerequisites,
    /// The same resource identifier occurs more than once among the prerequisites.
    DuplicatePrerequisites,
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    fn resource_id(value: &str) -> ResourceId {
        ResourceId::new(value.to_string()).expect("test resource ID must be valid")
    }

    fn route_id(value: &str) -> RouteId {
        RouteId::new(value.to_string()).expect("test route ID must be valid")
    }

    #[test]
    fn route_id_rejects_whitespace() {
        for value in ["", "   "] {
            assert!(matches!(
                RouteId::new(value.to_string()),
                Err(RouteError::InvalidId)
            ));
        }
    }

    #[test]
    fn route_accepts_valid_target_and_prerequisites() {
        let result = Route::new(
            route_id("email-login"),
            "Email login".to_string(),
            resource_id("email"),
            vec![resource_id("password-store"), resource_id("phone")],
        );

        let route = result.expect("valid route");
        assert_eq!(route.id().as_str(), "email-login");
        assert_eq!(route.label(), "Email login");
        assert_eq!(route.target().as_str(), "email");
        assert_eq!(
            route.prerequisites(),
            &[resource_id("password-store"), resource_id("phone")]
        );
    }

    #[test]
    fn route_rejects_empty_label() {
        let result = Route::new(
            route_id("email-login"),
            "   ".to_string(),
            resource_id("email"),
            vec![resource_id("phone")],
        );

        assert!(matches!(result, Err(RouteError::EmptyLabel)));
    }

    #[test]
    fn route_rejects_empty_prerequisites() {
        let result = Route::new(
            route_id("email-login"),
            "Email login".to_string(),
            resource_id("email"),
            Vec::new(),
        );

        assert!(matches!(result, Err(RouteError::EmptyPrerequisites)));
    }

    #[test]
    fn route_rejects_duplicate_prerequisite() {
        let result = Route::new(
            route_id("email-login"),
            "Email login".to_string(),
            resource_id("email"),
            vec![
                resource_id("phone"),
                resource_id("password-store"),
                resource_id("phone"),
            ],
        );

        assert!(matches!(result, Err(RouteError::DuplicatePrerequisites)));
    }

    #[test]
    fn route_accepts_target_as_prerequisite() {
        let result = Route::new(
            route_id("email-cycle"),
            "Circular email access".to_string(),
            resource_id("email"),
            vec![resource_id("email")],
        );

        assert!(result.is_ok());
    }

    #[test]
    fn and_both_prerequisites_stay_in_the_same_route() -> Result<(), RouteError> {
        // AND is represented by listing multiple ResourceIds inside a single
        // route's prerequisite Vec. The caller reads them back as a single
        // slice: every element in that slice is required together.
        let route = Route::new(
            route_id("email-login"),
            "Email login".to_string(),
            resource_id("email"),
            vec![resource_id("password-store"), resource_id("phone")],
        )?;

        let prereqs = route.prerequisites();
        assert_eq!(prereqs.len(), 2);
        assert!(prereqs.contains(&resource_id("password-store")));
        assert!(prereqs.contains(&resource_id("phone")));

        Ok(())
    }
}
