// Copyright (c) 2026 onyks-it
// SPDX-License-Identifier: AGPL-3.0-or-later
//!
//! Domain model for OpSecMap.

use std::fmt;

/// Creates a resource from textual identifier and label values.
///
/// # Errors
///
/// Returns [`ResourceError::InvalidId`] when `id` is empty or contains only
/// whitespace, or [`ResourceError::EmptyLabel`] when `label` is.
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
#[derive(Debug)]
pub struct Resource {
    /// Human-readable name; it is not used as a reference key.
    label: String,
    /// Stable identity used by routes and lookups.
    id: ResourceId,
    /// How completely the user has documented the routes to this resource.
    coverage: DocumentationCoverage,
}

impl Resource {
    /// Creates a resource.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::EmptyLabel`] when `label` is empty or contains
    /// only whitespace.
    pub fn new(id: ResourceId, label: String) -> Result<Resource, ResourceError> {
        if label.trim().is_empty() {
            return Err(ResourceError::EmptyLabel);
        }
        Ok(Resource {
            label,
            id,
            coverage: DocumentationCoverage::Unknown,
        })
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

    /// Updates the documentation coverage within the model.
    pub(super) fn set_coverage(&mut self, dc: DocumentationCoverage) {
        self.coverage = dc;
    }

    /// Updates a validated label within the model.
    pub(super) fn set_label(&mut self, label: String) {
        self.label = label;
    }

    /// Returns how completely the routes to this resource are documented.
    #[must_use]
    pub fn coverage(&self) -> DocumentationCoverage {
        self.coverage
    }
}

/// Errors produced while constructing resource values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceError {
    /// The identifier is empty or contains only whitespace.
    InvalidId,
    /// The human-readable label is empty or contains only whitespace.
    EmptyLabel,
}

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidId => f.write_str("resource ID is empty or blank"),
            Self::EmptyLabel => f.write_str("resource label is empty or blank"),
        }
    }
}

impl std::error::Error for ResourceError {}

/// The user's own statement about how completely a resource's routes are documented.
///
/// Coverage describes the map, not the real world: it never proves or disproves
/// access, and `Complete` is not a check against the service provider's rules.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DocumentationCoverage {
    /// The user has not stated how complete the documented routes are.
    Unknown,
    /// The user knows that some routes or details are missing.
    Incomplete,
    /// The user states that every known route to this resource is documented.
    Complete,
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{create_resource, DocumentationCoverage, Resource, ResourceError, ResourceId};

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
                Err(ResourceError::EmptyLabel)
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
            Err(ResourceError::EmptyLabel)
        ));

        let resource = create_resource("phone".to_string(), "Phone".to_string())
            .expect("both inputs are valid");
        assert_eq!(resource.id().as_str(), "phone");
        assert_eq!(resource.label(), "Phone");
    }

    #[test]
    fn new_resource_has_unknown_coverage() {
        let resource = create_resource("phone".to_string(), "Phone".to_string())
            .expect("both inputs are valid");

        assert_eq!(resource.coverage(), DocumentationCoverage::Unknown);
    }
}
