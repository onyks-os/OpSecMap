// Copyright (c) 2026 onyks-os
// SPDX-License-Identifier: AGPL-3.0-or-later
//!
//! Text rendering of evaluation results. Every function returns text; none prints.

use opsecmap::{
    AccessOrigin, BlockedRoute, DependencyMap, DocumentationCoverage, Evaluation, Resource,
    ResourceId, Scenario,
};

/// Summarizes the scenario's assumptions, such as
/// `Initial: recovery-sheet; excluded: phone`. An empty list reads `none`.
pub(crate) fn scenario_header(scenario: &Scenario) -> String {
    format!(
        "Initial: {}; excluded: {}",
        id_list(scenario.initially_available()),
        id_list(scenario.excluded())
    )
}

/// Joins IDs with `, `, or returns `none` for an empty list.
fn id_list(ids: &[ResourceId]) -> String {
    if ids.is_empty() {
        return "none".to_string();
    }
    ids.iter()
        .map(ResourceId::as_str)
        .collect::<Vec<&str>>()
        .join(", ")
}

/// Describes one resource with the access status recorded by the evaluation.
///
/// The order of the checks is the presentation rule: an exclusion is imposed
/// by the scenario, proven access wins over coverage, and coverage only
/// qualifies a resource without any access evidence.
pub(crate) fn resource_status_line(
    resource: &Resource,
    evaluation: &Evaluation,
    scenario: &Scenario,
) -> String {
    let id = resource.id();
    let status = if scenario.is_excluded(id) {
        "excluded by the scenario".to_string()
    } else if let Some(evidence) = evaluation.evidence_for(id) {
        match evidence.origin() {
            AccessOrigin::Initial => "available at start".to_string(),
            AccessOrigin::Derived { route } => format!("accessible via {}", route.as_str()),
        }
    } else {
        match evaluation.coverage_for(id) {
            Some(DocumentationCoverage::Complete) => {
                "not accessible with the documented procedures".to_string()
            }
            // None cannot occur for a resource taken from the evaluated map.
            Some(DocumentationCoverage::Unknown | DocumentationCoverage::Incomplete) | None => {
                "undetermined: documentation incomplete or not declared".to_string()
            }
        }
    };
    format!("{} ({}): {status}", id.as_str(), resource.label())
}

/// Describes a blocked route with the prerequisites it is missing.
pub(crate) fn blocked_route_line(blocked: &BlockedRoute) -> String {
    let missing: Vec<&str> = blocked.missing().iter().map(ResourceId::as_str).collect();
    format!(
        "{} -> {}: missing {}",
        blocked.route().as_str(),
        blocked.target().as_str(),
        missing.join(", ")
    )
}

/// Returns one line per map resource, in map order, followed by the blocked
/// routes or `Blocked routes: none`.
pub(crate) fn scenario_report(
    map: &DependencyMap,
    evaluation: &Evaluation,
    scenario: &Scenario,
) -> Vec<String> {
    let mut lines: Vec<String> = map
        .resources()
        .iter()
        .map(|resource| resource_status_line(resource, evaluation, scenario))
        .collect();
    if evaluation.blocked_routes().is_empty() {
        lines.push("Blocked routes: none".to_string());
    } else {
        lines.extend(evaluation.blocked_routes().iter().map(blocked_route_line));
    }
    lines
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{scenario_header, scenario_report};
    use opsecmap::{
        create_resource, DependencyMap, DocumentationCoverage, Evaluation, ResourceId, Route,
        RouteId, Scenario,
    };

    fn id(value: &str) -> ResourceId {
        ResourceId::new(value.to_string()).expect("test ID must be valid")
    }

    fn ids(values: &[&str]) -> Vec<ResourceId> {
        values.iter().map(|value| id(value)).collect()
    }

    /// The fictional map used by the demo: two routes to `email`, documented completely.
    fn example_map() -> DependencyMap {
        let mut map = DependencyMap::new();
        for (resource, label) in [
            ("email", "Test email"),
            ("password-store", "Test password manager"),
            ("phone", "Test phone"),
            ("recovery-sheet", "Test recovery sheet"),
        ] {
            let resource = create_resource(resource.to_string(), label.to_string())
                .expect("test resource must be valid");
            map.insert(resource).expect("test IDs must be distinct");
        }
        for (route, target, prerequisites) in [
            ("email-login", "email", &["password-store", "phone"][..]),
            ("email-recovery", "email", &["recovery-sheet"][..]),
        ] {
            let route = Route::new(
                RouteId::new(route.to_string()).expect("test route ID must be valid"),
                route.to_string(),
                id(target),
                ids(prerequisites),
            )
            .expect("test route must be valid");
            map.insert_route(route).expect("test route must fit map");
        }
        map.set_coverage(&id("email"), DocumentationCoverage::Complete)
            .expect("email exists");
        map
    }

    fn report(map: &DependencyMap, initial: &[&str], excluded: &[&str]) -> Vec<String> {
        let scenario = Scenario::new(ids(initial), ids(excluded)).expect("valid scenario");
        let evaluation = Evaluation::evaluate(map, &scenario).expect("scenario IDs exist");
        scenario_report(map, &evaluation, &scenario)
    }

    #[test]
    fn header_lists_both_assumptions_and_names_empty_lists() {
        let both = Scenario::new(ids(&["recovery-sheet", "seed"]), ids(&["phone"]))
            .expect("valid scenario");
        let empty = Scenario::new(Vec::new(), Vec::new()).expect("valid scenario");

        assert_eq!(
            scenario_header(&both),
            "Initial: recovery-sheet, seed; excluded: phone"
        );
        assert_eq!(scenario_header(&empty), "Initial: none; excluded: none");
    }

    #[test]
    fn report_shows_exclusion_initial_derived_and_undetermined_statuses() {
        let lines = report(&example_map(), &["recovery-sheet"], &["phone"]);

        assert_eq!(
            lines,
            [
                "email (Test email): accessible via email-recovery",
                "password-store (Test password manager): undetermined: documentation incomplete or not declared",
                "phone (Test phone): excluded by the scenario",
                "recovery-sheet (Test recovery sheet): available at start",
                "Blocked routes: none",
            ]
        );
    }

    #[test]
    fn report_shows_documented_dead_end_and_blocked_routes() {
        let lines = report(&example_map(), &["password-store"], &[]);

        assert_eq!(
            lines,
            [
                "email (Test email): not accessible with the documented procedures",
                "password-store (Test password manager): available at start",
                "phone (Test phone): undetermined: documentation incomplete or not declared",
                "recovery-sheet (Test recovery sheet): undetermined: documentation incomplete or not declared",
                "email-login -> email: missing phone",
                "email-recovery -> email: missing recovery-sheet",
            ]
        );
    }

    #[test]
    fn incomplete_coverage_is_reported_as_undetermined() {
        let mut map = example_map();
        map.set_coverage(&id("email"), DocumentationCoverage::Incomplete)
            .expect("email exists");

        let lines = report(&map, &["password-store"], &[]);

        assert_eq!(
            lines[0],
            "email (Test email): undetermined: documentation incomplete or not declared"
        );
    }

    #[test]
    fn accessible_resource_ignores_complete_coverage() {
        let lines = report(&example_map(), &["recovery-sheet"], &[]);

        assert_eq!(
            lines[0],
            "email (Test email): accessible via email-recovery"
        );
    }
}
