// Copyright (c) 2026 onyks-os
// SPDX-License-Identifier: AGPL-3.0-or-later
//! Map file format, version 1.
//!
//! The records below mirror the JSON file described in `docs/map-format.md`.
//! They hold raw, unvalidated values: a [`DependencyMap`](crate::DependencyMap)
//! is built from them only after every identifier and reference is checked.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{DependencyMap, DocumentationCoverage};

/// Value of the `format` field that identifies an OpSecMap map file.
const FORMAT: &str = "opsecmap-map";
/// The only format version this build reads and writes.
const VERSION: u32 = 1;

/// The whole map file: a header followed by resources and routes.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MapFile {
    /// Format name; always [`FORMAT`] in a valid file.
    format: String,
    /// Format version; always [`VERSION`] in a valid file.
    version: u32,
    /// Resources in map order.
    resources: Vec<ResourceRecord>,
    /// Routes in map order, after the resources they refer to.
    routes: Vec<RouteRecord>,
}

/// Copies a map into file records, keeping the map's order.
impl From<&DependencyMap> for MapFile {
    fn from(dm: &DependencyMap) -> Self {
        let mut resources = Vec::new();
        for r in dm.resources() {
            let resource = ResourceRecord {
                id: r.id().as_str().to_string(),
                label: r.label().to_string(),
                coverage: r.coverage().into(),
            };
            resources.push(resource);
        }

        let mut routes = Vec::new();
        for r in dm.routes() {
            let mut prerequisites = Vec::new();
            for p in r.prerequisites() {
                prerequisites.push(p.as_str().to_string());
            }
            let route = RouteRecord {
                id: r.id().as_str().to_string(),
                label: r.label().to_string(),
                target: r.target().as_str().to_string(),
                prerequisites,
            };
            routes.push(route);
        }
        Self {
            format: FORMAT.to_string(),
            version: VERSION,
            resources,
            routes,
        }
    }
}

/// Why a map could not be written to, or read from, the file format.
#[derive(Debug)]
pub(crate) enum MapFileError {
    /// The JSON could not be produced or parsed.
    Json(serde_json::Error),
}

impl fmt::Display for MapFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(e) => write!(f, "invalid map file: {e}"),
        }
    }
}

impl std::error::Error for MapFileError {}

/// One resource as stored in the file.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResourceRecord {
    /// Resource identifier, not yet validated.
    id: String,
    /// Human-readable label, not yet validated.
    label: String,
    /// How completely the resource's routes are documented in the map.
    coverage: CoverageRecord,
}

/// One access or recovery route as stored in the file.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RouteRecord {
    /// Route identifier, not yet validated.
    id: String,
    /// Human-readable label, not yet validated.
    label: String,
    /// Identifier of the resource this route grants access to.
    target: String,
    /// Identifiers of the resources all required to use the route.
    prerequisites: Vec<String>,
}

/// Documentation coverage as written in the file, in lowercase.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum CoverageRecord {
    /// Written as `"unknown"`.
    Unknown,
    /// Written as `"incomplete"`.
    Incomplete,
    /// Written as `"complete"`.
    Complete,
}

/// Maps each coverage level of the model to the record variant of the same name.
impl From<DocumentationCoverage> for CoverageRecord {
    fn from(dc: DocumentationCoverage) -> Self {
        match dc {
            DocumentationCoverage::Unknown => Self::Unknown,
            DocumentationCoverage::Incomplete => Self::Incomplete,
            DocumentationCoverage::Complete => Self::Complete,
        }
    }
}

/// Writes `map` as indented JSON in format version 1.
///
/// # Errors
///
/// Returns [`MapFileError::Json`] if serialization fails.
pub(crate) fn to_json(map: &DependencyMap) -> Result<String, MapFileError> {
    let file = MapFile::from(map);
    serde_json::to_string_pretty(&file).map_err(MapFileError::Json)
    // Equivalent to
    // match serde_json::to_string_pretty(&file) {
    //     Ok(text) => Ok(text),
    //     Err(e) => Err(MapFileError::Json(e)),
    // }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::to_json;
    use crate::{
        create_resource, DependencyMap, DocumentationCoverage, ResourceId, Route, RouteId,
    };

    type TestResult = Result<(), Box<dyn Error>>;

    fn resource_id(value: &str) -> Result<ResourceId, Box<dyn Error>> {
        Ok(ResourceId::new(value.to_string())?)
    }

    /// Two resources, one route and one `complete` coverage, as in `docs/map-format.md`.
    fn small_map() -> Result<DependencyMap, Box<dyn Error>> {
        let mut map = DependencyMap::new();
        map.insert(create_resource(
            "email".to_string(),
            "Test email".to_string(),
        )?)?;
        map.insert(create_resource(
            "phone".to_string(),
            "Test phone".to_string(),
        )?)?;
        map.insert_route(Route::new(
            RouteId::new("email-login".to_string())?,
            "Email login".to_string(),
            resource_id("email")?,
            vec![resource_id("phone")?],
        )?)?;
        map.set_coverage(&resource_id("email")?, DocumentationCoverage::Complete)?;
        Ok(map)
    }

    #[test]
    fn small_map_produces_the_expected_json() -> TestResult {
        let expected = r#"{
  "format": "opsecmap-map",
  "version": 1,
  "resources": [
    {
      "id": "email",
      "label": "Test email",
      "coverage": "complete"
    },
    {
      "id": "phone",
      "label": "Test phone",
      "coverage": "unknown"
    }
  ],
  "routes": [
    {
      "id": "email-login",
      "label": "Email login",
      "target": "email",
      "prerequisites": [
        "phone"
      ]
    }
  ]
}"#;

        assert_eq!(to_json(&small_map()?)?, expected);
        Ok(())
    }

    #[test]
    fn empty_map_has_empty_resources_and_routes() -> TestResult {
        let expected = r#"{
  "format": "opsecmap-map",
  "version": 1,
  "resources": [],
  "routes": []
}"#;

        assert_eq!(to_json(&DependencyMap::new())?, expected);
        Ok(())
    }

    #[test]
    fn same_map_always_produces_the_same_string() -> TestResult {
        let map = small_map()?;

        assert_eq!(to_json(&map)?, to_json(&map)?);
        Ok(())
    }

    #[test]
    fn json_follows_insertion_order_not_alphabetical_order() -> TestResult {
        let mut map = DependencyMap::new();
        for (id, label) in [("zeta", "Zeta"), ("alpha", "Alpha"), ("mid", "Mid")] {
            map.insert(create_resource(id.to_string(), label.to_string())?)?;
        }
        for (id, target) in [("to-mid", "mid"), ("to-alpha", "alpha")] {
            map.insert_route(Route::new(
                RouteId::new(id.to_string())?,
                "Route".to_string(),
                resource_id(target)?,
                vec![resource_id("zeta")?],
            )?)?;
        }

        let json = to_json(&map)?;
        let position = |needle: &str| json.find(needle).ok_or(needle.to_string());

        assert!(position(r#""id": "zeta""#)? < position(r#""id": "alpha""#)?);
        assert!(position(r#""id": "alpha""#)? < position(r#""id": "mid""#)?);
        assert!(position(r#""id": "to-mid""#)? < position(r#""id": "to-alpha""#)?);
        Ok(())
    }
}
