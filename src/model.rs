//! Domain model for OpSecMap.

// The struct is public (seen from outside the module),
// but the field is private
/// Resource id example
#[derive(Debug, PartialEq, Eq)]
pub struct ResourceId(String);

/// Implements methods for ResourceId
impl ResourceId {
    /// Creates a resource identifier.
    ///
    /// ## Errors
    ///
    /// Returns [`ResourceError::InvalidId`] when `value` is empty or contains
    /// only whitespace.
    pub fn new(value: String) -> Result<ResourceId, ResourceError> {
        if value.trim().is_empty() {
            return Err(ResourceError::InvalidId);
        }
        Ok(ResourceId(value))
    }

    /// Getter method for return the Id
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Defines a generic resource
pub struct Resource {
    /// Label for a given resource (ex.: My Phone)
    label: String,
    /// identifier
    id: ResourceId,
}

/// Implements method for Resource type
impl Resource {
    /// Creates a resource.
    ///
    /// ## Errors
    ///
    /// Returns [`ResourceError::InvalidId`] when `label` is empty or contains
    /// only whitespace.
    pub fn new(id: ResourceId, label: String) -> Result<Resource, ResourceError> {
        if label.trim().is_empty() {
            return Err(ResourceError::InvalidId);
        }
        Ok(Resource { label, id })
    }

    /// Returns the ResourceId
    #[must_use]
    pub fn id(&self) -> &ResourceId {
        &self.id
    }

    /// Returns the label
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }
}

/// Kind of errors
#[derive(Debug, PartialEq, Eq)]
pub enum ResourceError {
    /// Id is an empty string
    InvalidId,
}

/// DependencyMap of resources
pub struct DependencyMap {
    /// Resources of the inventory
    resources: Vec<Resource>,
}

/// Implementation of Inventory behaviour
impl DependencyMap {
    /// Instantiates a new Inventory object
    #[must_use]
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
        }
    }

    /// Returns the current resources within the Inventory
    #[must_use]
    pub fn resources(&self) -> &[Resource] {
        &self.resources
    }

    /// Finds a Resource given the ResourceId
    #[must_use]
    pub fn find(&self, id: &ResourceId) -> Option<&Resource> {
        self.resources
            .iter()
            .find(|&i| i.id.as_str() == id.as_str())
            .map(|v| v as _)
    }

    /// Inserts a new resource into the DependencyMap.
    ///
    /// ## Errors
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

    /// Checks if the DependencyMap is empty
    #[must_use]
    pub fn empty(&self) -> bool {
        self.resources().is_empty()
    }

    /// Changes the label of the resource identified by `id`.
    ///
    /// ## Errors
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
        DependencyMap::new()
    }
}

/// Errors occur within the DependencyMap
#[derive(Debug)]
pub enum MapError {
    /// Id is duplicated
    DuplicateId,
    /// Id is not found
    NotFound,
    /// Label is empty
    EmptyLabel,
}

/// Creates a resource from textual identifier and label values.
///
/// ## Errors
///
/// Returns [`ResourceError::InvalidId`] when either input is empty or contains
/// only whitespace.
pub fn create_resource(id: String, label: String) -> Result<Resource, ResourceError> {
    let resource_id = ResourceId::new(id)?;
    Resource::new(resource_id, label)
}

/// Tests
#[cfg(test)]
mod tests {
    use super::{create_resource, DependencyMap, MapError, ResourceError, ResourceId};

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
}
