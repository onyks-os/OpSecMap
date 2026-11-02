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
}

impl DependencyMap {
    /// Creates an empty dependency map.
    #[must_use]
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
        }
    }

    /// Returns all registered resources as a read-only slice.
    #[must_use]
    pub fn resources(&self) -> &[Resource] {
        &self.resources
    }

    /// Finds a resource by its stable identifier.
    #[must_use]
    pub fn find(&self, id: &ResourceId) -> Option<&Resource> {
        self.resources
            .iter()
            .find(|&i| i.id.as_str() == id.as_str())
            .map(|v| v as _)
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
        // TODO: change search algorithm
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
        create_resource, DependencyMap, MapError, ResourceError, ResourceId, Route, RouteError,
        RouteId,
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
}
