//! Domain model for OpSecMap.

/// Stable identifier for a resource.
///
/// The wrapped string is private so callers cannot bypass validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceId(String);

impl ResourceId {
    /// Creates a resource identifier.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::InvalidId`] when `value` is empty or contains
    /// only whitespace.
    pub fn new(value: String) -> Result<ResourceId, ResourceError> {
        if value.trim().is_empty() {
            return Err(ResourceError::InvalidId);
        }
        Ok(ResourceId(value))
    }

    /// Returns the identifier as borrowed text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A resource that can participate in access or recovery routes.
pub struct Resource {
    /// Human-readable name; it is not used as a reference key.
    label: String,
    /// Stable identity used by routes and lookups.
    id: ResourceId,
}

impl Resource {
    /// Creates a resource.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::InvalidId`] when `label` is empty or contains
    /// only whitespace.
    pub fn new(id: ResourceId, label: String) -> Result<Resource, ResourceError> {
        if label.trim().is_empty() {
            return Err(ResourceError::InvalidId);
        }
        Ok(Resource { label, id })
    }

    /// Returns the resource's stable identifier.
    #[must_use]
    pub fn id(&self) -> &ResourceId {
        &self.id
    }

    /// Returns the human-readable label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }
}

/// Errors produced while constructing resource values.
#[derive(Debug, PartialEq, Eq)]
pub enum ResourceError {
    /// An identifier or label is empty or contains only whitespace.
    InvalidId,
}

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
        let has_duplicate = prerequisites
            .iter()
            .enumerate()
            .any(|(index, prerequisite)| prerequisites[index + 1..].contains(prerequisite));
        if has_duplicate {
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
        self.resources
            .iter()
            .find(|&i| i.id.as_str() == id.as_str())
            .map(|v| v as _)
    }

    /// Finds a route by its stable identifier.
    #[must_use]
    pub fn find_route(&self, id: &RouteId) -> Option<&Route> {
        self.routes.iter().find(|&i| i.id() == id).map(|v| v as _)
    }

    /// Inserts a new resource into the DependencyMap.
    ///
    /// # Errors
    ///
    /// Returns [`MapError::DuplicateId`] when another resource already
    /// uses the same identifier.
    pub fn insert(&mut self, resource: Resource) -> Result<(), MapError> {
        let found = self.find(resource.id());
        if found.is_some() {
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
                r.label = label;
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

/// Creates a resource from textual identifier and label values.
///
/// # Errors
///
/// Returns [`ResourceError::InvalidId`] when either input is empty or contains
/// only whitespace.
pub fn create_resource(id: String, label: String) -> Result<Resource, ResourceError> {
    let resource_id = ResourceId::new(id)?;
    Resource::new(resource_id, label)
}

#[cfg(test)]
mod tests {
    use super::{
        create_resource, DependencyMap, MapError, Resource, ResourceError, ResourceId, Route,
        RouteError, RouteId,
    };

    fn resource_id(value: &str) -> ResourceId {
        ResourceId(value.to_string())
    }

    fn route_id(value: &str) -> RouteId {
        RouteId(value.to_string())
    }

    #[test]
    fn find_returns_existing_resource_and_none_for_missing_id() -> Result<(), ResourceError> {
        let mut inventory = DependencyMap::new();

        let phone = create_resource("phone".to_string(), "Pixel 9a".to_string())?;

        let email = create_resource("email".to_string(), "Email di prova".to_string())?;

        inventory.resources.push(phone);
        inventory.resources.push(email);

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
    fn route_id_rejects_whitespace() {
        let result = RouteId::new("   ".to_string());

        assert!(matches!(result, Err(RouteError::InvalidId)));
    }

    #[test]
    fn route_accepts_valid_target_and_prerequisites() {
        let result = Route::new(
            route_id("email-login"),
            "Email login".to_string(),
            resource_id("email"),
            vec![resource_id("password-store"), resource_id("phone")],
        );

        assert!(result.is_ok());
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
            vec![resource_id("phone"), resource_id("phone")],
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
    fn new_map_has_zero_resources_and_zero_routes() {
        let map = DependencyMap::new();
        assert_eq!(map.resources().len(), 0);
        assert_eq!(map.routes().len(), 0);
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
            let resource = Resource::new(ResourceId(id.to_string()), label.to_string())?;
            // IDs are distinct by construction; DuplicateId cannot occur here.
            let _ = map.insert(resource);
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
        let count_before = map.routes().len();

        let result = map.insert_route(duplicate);
        assert!(matches!(result, Err(MapError::DuplicateRouteId)));

        assert_eq!(map.routes().len(), count_before);

        Ok(())
    }

    #[test]
    fn insert_route_rejects_unknown_target_and_leaves_state_unchanged() -> Result<(), RouteError> {
        // A route whose target is absent from the map must return
        // UnknownRouteTarget carrying the problematic identifier, without
        // adding any route.
        let mut map = map_with_example_resources().map_err(|_| RouteError::InvalidId)?;
        let count_before = map.routes().len();

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

        assert_eq!(map.routes().len(), count_before);

        Ok(())
    }

    #[test]
    fn insert_route_rejects_unknown_prerequisite_and_leaves_state_unchanged(
    ) -> Result<(), RouteError> {
        // A route with a prerequisite absent from the map must return
        // UnknownRoutePrerequisite carrying the problematic identifier, without
        // adding any route.
        let mut map = map_with_example_resources().map_err(|_| RouteError::InvalidId)?;
        let count_before = map.routes().len();

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

        assert_eq!(map.routes().len(), count_before);

        Ok(())
    }

    // --- AND, OR, cycles and shared prerequisites ---

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
        let res_a = Resource::new(ResourceId("a".to_string()), "Resource A".to_string())
            .map_err(|_| RouteError::InvalidId)?;
        let res_b = Resource::new(ResourceId("b".to_string()), "Resource B".to_string())
            .map_err(|_| RouteError::InvalidId)?;
        let _ = map.insert(res_a);
        let _ = map.insert(res_b);

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
