// Copyright (c) 2026 onyks-os
// SPDX-License-Identifier: AGPL-3.0-or-later
//! Map file format, version 1.
//!
//! The records below mirror the JSON file described in `docs/map-format.md`.
//! They hold raw, unvalidated values: a [`DependencyMap`](crate::DependencyMap)
//! is built from them only after every identifier and reference is checked.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    DependencyMap, DocumentationCoverage, MapError, Resource, ResourceError, ResourceId, Route,
    RouteError, RouteId,
};

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
    fn from(map: &DependencyMap) -> Self {
        let mut resources = Vec::new();
        for resource in map.resources() {
            let record = ResourceRecord {
                id: resource.id().as_str().to_string(),
                label: resource.label().to_string(),
                coverage: resource.coverage().into(),
            };
            resources.push(record);
        }

        let mut routes = Vec::new();
        for route in map.routes() {
            let mut prerequisites = Vec::new();
            for prerequisite in route.prerequisites() {
                prerequisites.push(prerequisite.as_str().to_string());
            }
            let record = RouteRecord {
                id: route.id().as_str().to_string(),
                label: route.label().to_string(),
                target: route.target().as_str().to_string(),
                prerequisites,
            };
            routes.push(record);
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
pub enum MapFileError {
    /// The JSON could not be produced or parsed, or does not match the
    /// version 1 layout: a field is missing, unknown or of the wrong type.
    Json(serde_json::Error),
    /// The `format` field is not `"opsecmap-map"`: the file is not an OpSecMap map.
    NotAMapFile,
    /// The file declares a format version this build cannot read.
    UnsupportedVersion(u32),
    /// A resource in the file has an invalid ID or label.
    Resource(ResourceError),
    /// A route in the file has an invalid ID, label or prerequisite list.
    Route(RouteError),
    /// The file's resources and routes do not form a valid map, for example
    /// because of a duplicate ID or a reference to a missing resource.
    Map(MapError),
}

/// Describes the error in one line, including the underlying model or JSON error.
impl fmt::Display for MapFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(e) => write!(f, "invalid map file: {e}"),
            Self::NotAMapFile => write!(f, "not an OpSecMap map file"),
            Self::UnsupportedVersion(version) => write!(
                f,
                "unsupported map file version {version} (this build reads version {VERSION})"
            ),
            Self::Resource(e) => write!(f, "invalid resource in map file: {e}"),
            Self::Route(e) => write!(f, "invalid route in map file: {e}"),
            Self::Map(e) => write!(f, "inconsistent map file: {e}"),
        }
    }
}

impl std::error::Error for MapFileError {}

/// Lets `?` turn a resource error into [`MapFileError::Resource`].
impl From<ResourceError> for MapFileError {
    fn from(error: ResourceError) -> Self {
        Self::Resource(error)
    }
}

/// Lets `?` turn a route error into [`MapFileError::Route`].
impl From<RouteError> for MapFileError {
    fn from(error: RouteError) -> Self {
        Self::Route(error)
    }
}

/// Lets `?` turn a map error into [`MapFileError::Map`].
impl From<MapError> for MapFileError {
    fn from(error: MapError) -> Self {
        Self::Map(error)
    }
}

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
    fn from(coverage: DocumentationCoverage) -> Self {
        match coverage {
            DocumentationCoverage::Unknown => Self::Unknown,
            DocumentationCoverage::Incomplete => Self::Incomplete,
            DocumentationCoverage::Complete => Self::Complete,
        }
    }
}

/// Maps each record variant back to the model's coverage level of the same name.
impl From<CoverageRecord> for DocumentationCoverage {
    fn from(record: CoverageRecord) -> Self {
        match record {
            CoverageRecord::Unknown => Self::Unknown,
            CoverageRecord::Incomplete => Self::Incomplete,
            CoverageRecord::Complete => Self::Complete,
        }
    }
}

/// The two fields every version of the format starts with.
///
/// Without `deny_unknown_fields`, it reads any version's header and ignores the
/// rest, so the version can be checked before the strict [`MapFile`] parse.
#[derive(Deserialize)]
struct FileHeader {
    /// Format name, compared with [`FORMAT`].
    format: String,
    /// Format version, compared with [`VERSION`].
    version: u32,
}

/// Validates the records and builds the map in file order.
///
/// Every resource is inserted before any route, so a route may refer to any
/// resource in the file. Each check is the map model's own: the file cannot
/// produce a map that the public API would reject.
impl TryFrom<MapFile> for DependencyMap {
    type Error = MapFileError;

    fn try_from(file: MapFile) -> Result<Self, Self::Error> {
        let mut map = DependencyMap::new();
        for record in file.resources {
            let id = ResourceId::new(record.id)?;
            let resource = Resource::new(id.clone(), record.label)?;
            map.insert(resource)?;
            let coverage = DocumentationCoverage::from(record.coverage);
            map.set_coverage(&id, coverage)?;
        }
        for record in file.routes {
            let id = RouteId::new(record.id)?;
            let mut prerequisites = Vec::new();
            for prerequisite in record.prerequisites {
                let prerequisite = ResourceId::new(prerequisite)?;
                prerequisites.push(prerequisite);
            }
            let target = ResourceId::new(record.target)?;
            let route = Route::new(id, record.label, target, prerequisites)?;
            map.insert_route(route)?;
        }
        Ok(map)
    }
}

/// Writes `map` as indented JSON in format version 1.
///
/// The same map always produces the same string, in map order.
///
/// # Errors
///
/// Returns [`MapFileError::Json`] if serialization fails.
pub fn to_json(map: &DependencyMap) -> Result<String, MapFileError> {
    let file = MapFile::from(map);
    serde_json::to_string_pretty(&file).map_err(MapFileError::Json)
    // Equivalent to
    // match serde_json::to_string_pretty(&file) {
    //     Ok(text) => Ok(text),
    //     Err(e) => Err(MapFileError::Json(e)),
    // }
}

/// Reads a map from JSON in format version 1.
///
/// The header is checked first, so a file from another version is reported as
/// [`MapFileError::UnsupportedVersion`] rather than as a JSON error.
///
/// # Errors
///
/// - [`MapFileError::Json`] if `text` is not valid JSON or does not match the
///   version 1 layout.
/// - [`MapFileError::NotAMapFile`] if the `format` field is wrong.
/// - [`MapFileError::UnsupportedVersion`] if the version is not 1.
/// - [`MapFileError::Resource`], [`MapFileError::Route`] or [`MapFileError::Map`]
///   if the content breaks a rule of the map model.
pub fn from_json(text: &str) -> Result<DependencyMap, MapFileError> {
    let header: FileHeader = serde_json::from_str(text).map_err(MapFileError::Json)?;
    if header.format != FORMAT {
        return Err(MapFileError::NotAMapFile);
    }
    if header.version != VERSION {
        return Err(MapFileError::UnsupportedVersion(header.version));
    }
    let file: MapFile = serde_json::from_str(text).map_err(MapFileError::Json)?;
    DependencyMap::try_from(file)
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
