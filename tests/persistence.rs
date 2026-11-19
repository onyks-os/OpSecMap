// Copyright (c) 2026 onyks-os
// SPDX-License-Identifier: AGPL-3.0-or-later
//!
//! Reading and writing the map file format through the public API.
//!
//! Every map here is fictional. The fixture is the same map the CLI demo builds.

use std::error::Error;

use opsecmap::{
    create_resource, from_json, to_json, DependencyMap, DocumentationCoverage, Evaluation,
    MapError, MapFileError, ResourceError, ResourceId, Route, RouteError, RouteId, Scenario,
};

type TestResult = Result<(), Box<dyn Error>>;

const FIXTURE: &str = include_str!("fixtures/phone-lost.json");

fn resource_id(value: &str) -> Result<ResourceId, Box<dyn Error>> {
    Ok(ResourceId::new(value.to_string())?)
}

/// The demo map, built through the library API as `opsecmap --demo` builds it.
fn demo_map() -> Result<DependencyMap, Box<dyn Error>> {
    let mut map = DependencyMap::new();
    for (id, label) in [
        ("email", "Test email"),
        ("password-store", "Test password manager"),
        ("phone", "Test phone"),
        ("recovery-sheet", "Test recovery sheet"),
    ] {
        map.insert(create_resource(id.to_string(), label.to_string())?)?;
    }
    map.insert_route(Route::new(
        RouteId::new("email-login".to_string())?,
        "Email login".to_string(),
        resource_id("email")?,
        vec![resource_id("password-store")?, resource_id("phone")?],
    )?)?;
    map.insert_route(Route::new(
        RouteId::new("email-recovery".to_string())?,
        "Email recovery".to_string(),
        resource_id("email")?,
        vec![resource_id("recovery-sheet")?],
    )?)?;
    map.set_coverage(&resource_id("email")?, DocumentationCoverage::Complete)?;
    Ok(map)
}

/// A version 1 file with one resource per entry and no routes.
fn file_with_resources(resources: &str) -> String {
    format!(
        r#"{{"format": "opsecmap-map", "version": 1, "resources": [{resources}], "routes": []}}"#
    )
}

/// A version 1 file with resources `a` and `b` and the given routes.
fn file_with_routes(routes: &str) -> String {
    format!(
        r#"{{"format": "opsecmap-map", "version": 1,
            "resources": [
                {{"id": "a", "label": "A", "coverage": "unknown"}},
                {{"id": "b", "label": "B", "coverage": "unknown"}}
            ],
            "routes": [{routes}]}}"#
    )
}

#[test]
fn round_trip_gives_the_same_string() -> TestResult {
    let first = to_json(&demo_map()?)?;
    let second = to_json(&from_json(&first)?)?;

    assert_eq!(first, second);
    Ok(())
}

#[test]
fn fixture_is_the_demo_map_as_written_by_to_json() -> TestResult {
    // The file ends with a newline; `to_json` does not write one.
    assert_eq!(FIXTURE.trim_end(), to_json(&demo_map()?)?);
    Ok(())
}

#[test]
fn fixture_gives_the_demo_result_for_phone_lost() -> TestResult {
    let phone_lost = Scenario::new(
        vec![resource_id("recovery-sheet")?],
        vec![resource_id("phone")?],
    )?;
    let from_file = Evaluation::evaluate(&from_json(FIXTURE)?, &phone_lost)?;
    let from_demo = Evaluation::evaluate(&demo_map()?, &phone_lost)?;

    assert_eq!(from_file, from_demo);
    assert!(from_file.is_accessible(&resource_id("email")?));
    Ok(())
}

#[test]
fn truncated_json_is_a_json_error() {
    let truncated = &FIXTURE[..FIXTURE.len() / 2];

    assert!(matches!(from_json(truncated), Err(MapFileError::Json(_))));
}

#[test]
fn other_format_is_not_a_map_file() {
    let text = r#"{"format": "altro", "version": 1, "resources": [], "routes": []}"#;

    assert!(matches!(from_json(text), Err(MapFileError::NotAMapFile)));
}

#[test]
fn version_2_is_unsupported() {
    let text = r#"{"format": "opsecmap-map", "version": 2, "resources": [], "routes": []}"#;

    assert!(matches!(
        from_json(text),
        Err(MapFileError::UnsupportedVersion(2))
    ));
}

#[test]
fn version_2_with_new_fields_is_unsupported_not_json() {
    // A future version may add fields; the header check must come first.
    let text = r#"{"format": "opsecmap-map", "version": 2, "tags": ["work"], "groups": {}}"#;

    assert!(matches!(
        from_json(text),
        Err(MapFileError::UnsupportedVersion(2))
    ));
}

#[test]
fn empty_resource_id_is_a_resource_error() {
    let text = file_with_resources(r#"{"id": "", "label": "Empty", "coverage": "unknown"}"#);

    assert!(matches!(
        from_json(&text),
        Err(MapFileError::Resource(ResourceError::InvalidId))
    ));
}

#[test]
fn route_without_prerequisites_is_a_route_error() {
    let text = file_with_routes(r#"{"id": "r", "label": "R", "target": "a", "prerequisites": []}"#);

    assert!(matches!(
        from_json(&text),
        Err(MapFileError::Route(RouteError::EmptyPrerequisites))
    ));
}

#[test]
fn missing_prerequisite_is_a_map_error() {
    let text = file_with_routes(
        r#"{"id": "r", "label": "R", "target": "a", "prerequisites": ["missing"]}"#,
    );

    assert!(matches!(
        from_json(&text),
        Err(MapFileError::Map(MapError::UnknownRoutePrerequisite(id))) if id.as_str() == "missing"
    ));
}

#[test]
fn duplicate_resource_is_a_map_error() {
    let resource = r#"{"id": "a", "label": "A", "coverage": "unknown"}"#;
    let text = file_with_resources(&format!("{resource}, {resource}"));

    assert!(matches!(
        from_json(&text),
        Err(MapFileError::Map(MapError::DuplicateId(id))) if id.as_str() == "a"
    ));
}

#[test]
fn unknown_field_is_a_json_error() {
    let text =
        file_with_resources(r#"{"id": "a", "label": "A", "coverage": "unknown", "colour": "red"}"#);

    assert!(matches!(from_json(&text), Err(MapFileError::Json(_))));
}

#[test]
fn missing_field_is_a_json_error() {
    let text = file_with_resources(r#"{"id": "a", "label": "A"}"#);

    assert!(matches!(from_json(&text), Err(MapFileError::Json(_))));
}

#[test]
fn error_message_names_the_underlying_problem() {
    let text = r#"{"format": "opsecmap-map", "version": 2, "resources": [], "routes": []}"#;
    let message = from_json(text).map(|_| ()).err().map(|e| e.to_string());

    assert_eq!(
        message.as_deref(),
        Some("unsupported map file version 2 (this build reads version 1)")
    );
}

#[test]
fn routes_written_before_resources_are_accepted() {
    let text = r#"{"routes": [{"id": "r", "label": "R", "target": "a", "prerequisites": ["b"]}],
        "resources": [
            {"id": "a", "label": "A", "coverage": "unknown"},
            {"id": "b", "label": "B", "coverage": "unknown"}
        ],
        "version": 1, "format": "opsecmap-map"}"#;

    assert!(from_json(text).is_ok());
}
