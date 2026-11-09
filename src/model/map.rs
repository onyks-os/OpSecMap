// Copyright (c) 2026 onyks-it
// SPDX-License-Identifier: AGPL-3.0-or-later
//!
//! Domain model for OpSecMap.

use super::{Resource, ResourceId, Route, RouteId};

/// A validated dependency map containing resources and their relationships.
pub struct DependencyMap {
    /// Resources currently registered in the map.
    resources: Vec<Resource>,
    /// Access and recovery routes that link resources together.
    routes: Vec<Route>,
}

impl DependencyMap {
    /// Creates an empty dependency map.
    #[must_use]
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
            routes: Vec::new(),
        }
    }

    /// Returns all registered resources as a read-only slice.
    #[must_use]
    pub fn resources(&self) -> &[Resource] {
        &self.resources
    }

    /// Returns all registered routes as a read-only slice.
    #[must_use]
    pub fn routes(&self) -> &[Route] {
        &self.routes
    }

    /// Finds a resource by its stable identifier.
    #[must_use]
    pub fn find(&self, id: &ResourceId) -> Option<&Resource> {
        self.resources.iter().find(|resource| resource.id() == id)
    }

    /// Finds a route by its stable identifier.
    #[must_use]
    pub fn find_route(&self, id: &RouteId) -> Option<&Route> {
        self.routes.iter().find(|route| route.id() == id)
    }

    /// Inserts a new resource into the DependencyMap.
    ///
    /// # Errors
    ///
    /// Returns [`MapError::DuplicateId`] when another resource already
    /// uses the same identifier.
    pub fn insert(&mut self, resource: Resource) -> Result<(), MapError> {
        if self.find(resource.id()).is_some() {
            return Err(MapError::DuplicateId);
        }
        self.resources.push(resource);
        Ok(())
    }

    /// Inserts a validated route into the map, enforcing referential integrity.
    ///
    /// # Errors
    ///
    /// Returns [`MapError::DuplicateRouteId`] when a route with the same identifier
    /// already exists, [`MapError::UnknownRouteTarget`] when the target resource is
    /// not present in the map, or [`MapError::UnknownRoutePrerequisite`] when any
    /// prerequisite resource is not present in the map. On error the map is unchanged.
    pub fn insert_route(&mut self, route: Route) -> Result<(), MapError> {
        if self.find_route(route.id()).is_some() {
            return Err(MapError::DuplicateRouteId);
        }
        let target = route.target();
        if self.find(target).is_none() {
            return Err(MapError::UnknownRouteTarget(target.clone()));
        }
        for p in route.prerequisites() {
            if self.find(p).is_none() {
                return Err(MapError::UnknownRoutePrerequisite(p.clone()));
            }
        }
        self.routes.push(route);
        Ok(())
    }

    /// Returns `true` when the map contains no resources.
    #[must_use]
    pub fn empty(&self) -> bool {
        self.resources().is_empty()
    }

    /// Changes the label of the resource identified by `id`.
    ///
    /// # Errors
    ///
    /// Returns [`MapError::EmptyLabel`] when `label` is empty or contains
    /// only whitespace. Returns [`MapError::NotFound`] when `id` is not
    /// present in the DependencyMap.
    pub fn rename(&mut self, id: &ResourceId, label: String) -> Result<(), MapError> {
        if label.trim().is_empty() {
            return Err(MapError::EmptyLabel);
        }
        for r in &mut self.resources {
            if *id == *r.id() {
                r.set_label(label);
                return Ok(());
            }
        }
        Err(MapError::NotFound)
    }
}

impl Default for DependencyMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors produced by dependency-map operations.
#[derive(Debug)]
pub enum MapError {
    /// Another resource already uses the requested identifier.
    DuplicateId,
    /// The requested resource does not exist.
    NotFound,
    /// A replacement label is empty or contains only whitespace.
    EmptyLabel,
    /// Another route already uses the requested identifier.
    DuplicateRouteId,
    /// The target resource of a route does not exist in the map.
    UnknownRouteTarget(ResourceId),
    /// A prerequisite resource of a route does not exist in the map.
    UnknownRoutePrerequisite(ResourceId),
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::model::{create_resource, ResourceError, RouteError};

    fn resource_id(value: &str) -> ResourceId {
        ResourceId::new(value.to_string()).expect("test resource ID must be valid")
    }

    fn route_id(value: &str) -> RouteId {
        RouteId::new(value.to_string()).expect("test route ID must be valid")
    }

    fn route_snapshot(map: &DependencyMap) -> Vec<(String, String, String, Vec<String>)> {
        map.routes()
            .iter()
            .map(|route| {
                (
                    route.id().as_str().to_string(),
                    route.label().to_string(),
                    route.target().as_str().to_string(),
                    route
                        .prerequisites()
                        .iter()
                        .map(|id| id.as_str().to_string())
                        .collect(),
                )
            })
            .collect()
    }

    #[test]
    fn find_returns_existing_resource_and_none_for_missing_id() -> Result<(), ResourceError> {
        let mut inventory = DependencyMap::new();

        let phone = create_resource("phone".to_string(), "Pixel 9a".to_string())?;

        let email = create_resource("email".to_string(), "Email di prova".to_string())?;

        assert!(inventory.insert(phone).is_ok());
        assert!(inventory.insert(email).is_ok());

        let existing_id = ResourceId::new("phone".to_string())?;
        let missing_id = ResourceId::new("missing".to_string())?;

        let found = inventory.find(&existing_id);

        assert!(matches!(
            found,
            Some(resource) if resource.label() == "Pixel 9a"
        ));

        assert!(inventory.find(&missing_id).is_none());

        Ok(())
    }

    #[test]
    fn insert_rejects_duplicate_without_changing_inventory() -> Result<(), ResourceError> {
        let mut inventory = DependencyMap::new();

        let original = create_resource("phone".to_string(), "Pixel 9a".to_string())?;

        let duplicate = create_resource("phone".to_string(), "Altro telefono".to_string())?;

        assert!(inventory.insert(original).is_ok());

        let count_before_error = inventory.resources().len();

        let result = inventory.insert(duplicate);

        assert!(matches!(result, Err(MapError::DuplicateId)));

        assert_eq!(inventory.resources().len(), count_before_error);

        let phone_id = ResourceId::new("phone".to_string())?;

        assert!(matches!(
            inventory.find(&phone_id),
            Some(resource) if resource.label() == "Pixel 9a"
        ));

        Ok(())
    }

    #[test]
    fn insert_allows_same_label_with_different_ids() -> Result<(), ResourceError> {
        let mut inventory = DependencyMap::new();

        let first = create_resource("phone-primary".to_string(), "Pixel 9a".to_string())?;

        let second = create_resource("phone-backup".to_string(), "Pixel 9a".to_string())?;

        assert!(inventory.insert(first).is_ok());
        assert!(inventory.insert(second).is_ok());

        assert_eq!(inventory.resources().len(), 2);

        let first_id = ResourceId::new("phone-primary".to_string())?;

        let second_id = ResourceId::new("phone-backup".to_string())?;

        assert!(inventory.find(&first_id).is_some());
        assert!(inventory.find(&second_id).is_some());

        Ok(())
    }

    #[test]
    fn new_map_has_zero_resources_and_zero_routes() {
        let map = DependencyMap::new();
        assert_eq!(map.resources().len(), 0);
        assert_eq!(map.routes().len(), 0);
        assert!(map.empty());
    }

    #[test]
    fn default_map_is_empty_and_becomes_nonempty_after_insert() {
        let mut map = DependencyMap::default();
        assert!(map.empty());
        let phone =
            Resource::new(resource_id("phone"), "Phone".to_string()).expect("valid resource");
        assert!(map.insert(phone).is_ok());
        assert!(!map.empty());
    }

    #[test]
    fn rename_rejects_blank_label_without_changing_resource() {
        let mut map = DependencyMap::default();
        let phone =
            Resource::new(resource_id("phone"), "Original".to_string()).expect("valid resource");
        assert!(map.insert(phone).is_ok());

        for label in ["", " \t\n "] {
            assert!(matches!(
                map.rename(&resource_id("phone"), label.to_string()),
                Err(MapError::EmptyLabel)
            ));
            assert_eq!(
                map.find(&resource_id("phone")).map(Resource::label),
                Some("Original")
            );
        }
    }

    #[test]
    fn rename_rejects_absent_id_without_changing_other_resources() {
        let mut map = DependencyMap::default();
        let phone =
            Resource::new(resource_id("phone"), "Original".to_string()).expect("valid resource");
        assert!(map.insert(phone).is_ok());

        assert!(matches!(
            map.rename(&resource_id("missing"), "Updated".to_string()),
            Err(MapError::NotFound)
        ));
        assert_eq!(map.resources().len(), 1);
        assert_eq!(
            map.find(&resource_id("phone")).map(Resource::label),
            Some("Original")
        );
    }

    #[test]
    fn find_route_returns_none_for_absent_id() -> Result<(), RouteError> {
        let map = DependencyMap::new();
        let absent = RouteId::new("email-login".to_string())?;
        assert!(map.find_route(&absent).is_none());
        Ok(())
    }

    // --- insert_route ---

    /// Returns a map pre-populated with the four example resources.
    ///
    /// All inputs are statically known to be valid; any error here indicates
    /// a bug in the test setup, so the function propagates it as `ResourceError`.
    fn map_with_example_resources() -> Result<DependencyMap, ResourceError> {
        let mut map = DependencyMap::new();
        for (id, label) in [
            ("email", "Test email"),
            ("password-store", "Test password manager"),
            ("phone", "Test phone"),
            ("recovery-sheet", "Test recovery sheet"),
        ] {
            let resource = Resource::new(resource_id(id), label.to_string())?;
            assert!(map.insert(resource).is_ok(), "test fixture must be valid");
        }
        Ok(map)
    }

    #[test]
    fn insert_route_accepts_valid_route() -> Result<(), RouteError> {
        // A route whose target and all prerequisites exist must be accepted
        // and must be retrievable afterwards via find_route.
        let mut map = map_with_example_resources().map_err(|_| RouteError::InvalidId)?;

        let route = Route::new(
            route_id("email-login"),
            "Email login".to_string(),
            resource_id("email"),
            vec![resource_id("password-store"), resource_id("phone")],
        )?;

        assert!(map.insert_route(route).is_ok());
        assert_eq!(map.routes().len(), 1);

        let id = route_id("email-login");
        assert!(map.find_route(&id).is_some());

        Ok(())
    }

    #[test]
    fn insert_route_rejects_duplicate_id_and_leaves_state_unchanged() -> Result<(), RouteError> {
        // A second insertion sharing the same RouteId must return DuplicateRouteId
        // and must not change the number or content of stored routes.
        let mut map = map_with_example_resources().map_err(|_| RouteError::InvalidId)?;

        let first = Route::new(
            route_id("email-login"),
            "Email login".to_string(),
            resource_id("email"),
            vec![resource_id("password-store")],
        )?;
        let duplicate = Route::new(
            route_id("email-login"),
            "Duplicate".to_string(),
            resource_id("email"),
            vec![resource_id("phone")],
        )?;

        assert!(map.insert_route(first).is_ok());
        let before = route_snapshot(&map);

        let result = map.insert_route(duplicate);
        assert!(matches!(result, Err(MapError::DuplicateRouteId)));

        assert_eq!(route_snapshot(&map), before);

        Ok(())
    }

    #[test]
    fn insert_route_rejects_unknown_target_and_leaves_state_unchanged() -> Result<(), RouteError> {
        // A route whose target is absent from the map must return
        // UnknownRouteTarget carrying the problematic identifier, without
        // adding any route.
        let mut map = map_with_example_resources().map_err(|_| RouteError::InvalidId)?;
        let existing = Route::new(
            route_id("email-recovery"),
            "Email recovery".to_string(),
            resource_id("email"),
            vec![resource_id("recovery-sheet")],
        )?;
        assert!(map.insert_route(existing).is_ok());
        let before = route_snapshot(&map);

        let route = Route::new(
            route_id("broken-login"),
            "Route with missing target".to_string(),
            resource_id("ghost"),
            vec![resource_id("phone")],
        )?;

        let result = map.insert_route(route);
        assert!(matches!(
            result,
            Err(MapError::UnknownRouteTarget(ref id)) if id.as_str() == "ghost"
        ));

        assert_eq!(route_snapshot(&map), before);

        Ok(())
    }

    #[test]
    fn insert_route_rejects_unknown_prerequisite_and_leaves_state_unchanged(
    ) -> Result<(), RouteError> {
        // A route with a prerequisite absent from the map must return
        // UnknownRoutePrerequisite carrying the problematic identifier, without
        // adding any route.
        let mut map = map_with_example_resources().map_err(|_| RouteError::InvalidId)?;
        let existing = Route::new(
            route_id("email-recovery"),
            "Email recovery".to_string(),
            resource_id("email"),
            vec![resource_id("recovery-sheet")],
        )?;
        assert!(map.insert_route(existing).is_ok());
        let before = route_snapshot(&map);

        let route = Route::new(
            route_id("email-login"),
            "Email login with missing prerequisite".to_string(),
            resource_id("email"),
            vec![resource_id("password-store"), resource_id("ghost-key")],
        )?;

        let result = map.insert_route(route);
        assert!(matches!(
            result,
            Err(MapError::UnknownRoutePrerequisite(ref id)) if id.as_str() == "ghost-key"
        ));

        assert_eq!(route_snapshot(&map), before);

        Ok(())
    }

    // --- AND, OR, cycles and shared prerequisites ---

    #[test]
    fn or_two_routes_with_same_target_are_both_accepted() -> Result<(), RouteError> {
        // OR is represented by two separate Route values that share the same
        // target ResourceId but carry distinct RouteIds. The map accepts both
        // because uniqueness is enforced on RouteId, not on the target.
        let mut map = map_with_example_resources().map_err(|_| RouteError::InvalidId)?;

        let login = Route::new(
            route_id("email-login"),
            "Email login".to_string(),
            resource_id("email"),
            vec![resource_id("password-store"), resource_id("phone")],
        )?;
        let recovery = Route::new(
            route_id("email-recovery"),
            "Email recovery".to_string(),
            resource_id("email"),
            vec![resource_id("recovery-sheet")],
        )?;

        assert!(map.insert_route(login).is_ok());
        assert!(map.insert_route(recovery).is_ok());
        assert_eq!(map.routes().len(), 2);

        // Both routes point to the same target.
        let routes_to_email: Vec<_> = map
            .routes()
            .iter()
            .filter(|r| r.target() == &resource_id("email"))
            .collect();
        assert_eq!(routes_to_email.len(), 2);

        Ok(())
    }

    #[test]
    fn cycle_between_two_resources_is_accepted() -> Result<(), RouteError> {
        // A cycle (a → b and b → a) is a valid structure: the map only checks
        // that referenced resources exist, not whether the cycle is reachable.
        let mut map = DependencyMap::new();
        let res_a = Resource::new(resource_id("a"), "Resource A".to_string())
            .map_err(|_| RouteError::InvalidId)?;
        let res_b = Resource::new(resource_id("b"), "Resource B".to_string())
            .map_err(|_| RouteError::InvalidId)?;
        assert!(map.insert(res_a).is_ok());
        assert!(map.insert(res_b).is_ok());

        let a_to_b = Route::new(
            route_id("a-to-b"),
            "A unlocks B".to_string(),
            resource_id("b"),
            vec![resource_id("a")],
        )?;
        let b_to_a = Route::new(
            route_id("b-to-a"),
            "B unlocks A".to_string(),
            resource_id("a"),
            vec![resource_id("b")],
        )?;

        assert!(map.insert_route(a_to_b).is_ok());
        assert!(map.insert_route(b_to_a).is_ok());
        assert_eq!(map.routes().len(), 2);

        Ok(())
    }

    #[test]
    fn shared_prerequisite_can_appear_in_multiple_routes() -> Result<(), RouteError> {
        // The same ResourceId may appear as a prerequisite in separate routes.
        // Each route owns its own copy of the identifier value.
        let mut map = map_with_example_resources().map_err(|_| RouteError::InvalidId)?;

        let login = Route::new(
            route_id("email-login"),
            "Email login".to_string(),
            resource_id("email"),
            vec![resource_id("password-store"), resource_id("phone")],
        )?;
        let recovery = Route::new(
            route_id("email-recovery"),
            "Email recovery".to_string(),
            resource_id("email"),
            vec![resource_id("recovery-sheet")],
        )?;
        // A second hypothetical route that also requires "phone".
        let phone_only = Route::new(
            route_id("email-phone-only"),
            "Email via phone OTP".to_string(),
            resource_id("email"),
            vec![resource_id("phone")],
        )?;

        assert!(map.insert_route(login).is_ok());
        assert!(map.insert_route(recovery).is_ok());
        assert!(map.insert_route(phone_only).is_ok());

        // "phone" appears in two distinct routes.
        let routes_with_phone: Vec<_> = map
            .routes()
            .iter()
            .filter(|r| r.prerequisites().contains(&resource_id("phone")))
            .collect();
        assert_eq!(routes_with_phone.len(), 2);

        Ok(())
    }

    #[test]
    fn renaming_resource_after_inserting_routes_preserves_reference() -> Result<(), RouteError> {
        // Renaming a resource only updates its human-readable label; existing
        // routes continue to reference the resource by its stable identifier.
        let mut map = map_with_example_resources().map_err(|_| RouteError::InvalidId)?;

        let login = Route::new(
            route_id("email-login"),
            "Email login".to_string(),
            resource_id("email"),
            vec![resource_id("password-store"), resource_id("phone")],
        )?;
        let recovery = Route::new(
            route_id("email-recovery"),
            "Email recovery".to_string(),
            resource_id("email"),
            vec![resource_id("recovery-sheet")],
        )?;

        assert!(map.insert_route(login).is_ok());
        assert!(map.insert_route(recovery).is_ok());

        // Rename phone after routes have been registered.
        let phone_id = resource_id("phone");
        assert!(map.rename(&phone_id, "Backup phone".to_string()).is_ok());

        // Verify that the label was updated in the map.
        assert_eq!(
            map.find(&phone_id).map(Resource::label),
            Some("Backup phone")
        );

        // Verify that email-login still references the stable phone identifier.
        let email_login = map
            .find_route(&route_id("email-login"))
            .ok_or(RouteError::InvalidId)?;

        assert!(email_login.prerequisites().contains(&phone_id));

        Ok(())
    }
}
