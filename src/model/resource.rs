// Copyright (c) 2026 onyks-it
// SPDX-License-Identifier: AGPL-3.0-or-later
//!
//! Domain model for OpSecMap.

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

/// Stable identifier for a resource.
///
/// The wrapped string is private so callers cannot bypass validation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

    /// Updates a validated label within the model.
    pub(super) fn set_label(&mut self, label: String) {
        self.label = label;
    }
}

/// Errors produced while constructing resource values.
#[derive(Debug, PartialEq, Eq)]
pub enum ResourceError {
    /// An identifier or label is empty or contains only whitespace.
    InvalidId,
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{create_resource, Resource, ResourceError, ResourceId};

    #[test]
    fn resource_id_rejects_empty_and_whitespace_only_values() {
        for value in ["", " \t\n "] {
            assert!(matches!(
                ResourceId::new(value.to_string()),
                Err(ResourceError::InvalidId)
            ));
        }
    }

    #[test]
    fn valid_resource_preserves_id_and_label() {
        let id = ResourceId::new("phone".to_string()).expect("valid ID");
        let resource = Resource::new(id, "Primary phone".to_string()).expect("valid label");

        assert_eq!(resource.id().as_str(), "phone");
        assert_eq!(resource.label(), "Primary phone");
    }

    #[test]
    fn resource_rejects_empty_and_whitespace_only_labels() {
        for label in ["", " \t\n "] {
            let id = ResourceId::new("phone".to_string()).expect("valid ID");
            assert!(matches!(
                Resource::new(id, label.to_string()),
                Err(ResourceError::InvalidId)
            ));
        }
    }

    #[test]
    fn create_resource_validates_both_inputs() {
        assert!(matches!(
            create_resource(" ".to_string(), "Phone".to_string()),
            Err(ResourceError::InvalidId)
        ));
        assert!(matches!(
            create_resource("phone".to_string(), " ".to_string()),
            Err(ResourceError::InvalidId)
        ));

        let resource = create_resource("phone".to_string(), "Phone".to_string())
            .expect("both inputs are valid");
        assert_eq!(resource.id().as_str(), "phone");
        assert_eq!(resource.label(), "Phone");
    }
}
