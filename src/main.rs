// Copyright (c) 2026 onyks-it
// SPDX-License-Identifier: AGPL-3.0-or-later
//!
//! Command line entry point for OpSecMap.
//!
//! This binary parses arguments and formats output; the logic lives in the
//! library crate so it stays testable without a terminal.

use opsecmap::{
    create_resource, AccessOrigin, BlockedRoute, DependencyMap, DocumentationCoverage, Evaluation,
    Resource, ResourceId, Route, RouteId, Scenario, VERSION,
};
use std::process::ExitCode;

/// Example scenarios: title, initially available IDs and excluded IDs.
const SCENARIOS: [(&str, &[&str], &[&str]); 2] = [
    ("Phone lost", &["recovery-sheet"], &["phone"]),
    ("Only password manager", &["password-store"], &[]),
];

/// Validates raw strings as resource IDs, printing the first error.
fn parse_ids(values: &[&str]) -> Option<Vec<ResourceId>> {
    let mut ids = Vec::with_capacity(values.len());
    for value in values {
        match ResourceId::new((*value).to_string()) {
            Ok(id) => ids.push(id),
            Err(e) => {
                eprintln!("Cannot create ResourceId \"{value}\": {e}");
                return None;
            }
        }
    }
    Some(ids)
}

/// Inserts a resource built from raw strings, printing an error and returning
/// `false` on failure.
fn insert(map: &mut DependencyMap, id: &str, label: &str) -> bool {
    match create_resource(id.to_string(), label.to_string()) {
        Ok(resource) => match map.insert(resource) {
            Ok(()) => true,
            Err(e) => {
                eprintln!("Cannot insert resource \"{id}\": {e}");
                false
            }
        },
        Err(e) => {
            eprintln!("Cannot create resource \"{id}\": {e}");
            false
        }
    }
}

/// Inserts a route, printing an error and returning `false` on failure.
fn insert_route(
    map: &mut DependencyMap,
    route_id: &str,
    label: &str,
    target: &str,
    prereqs: &[&str],
) -> bool {
    let rid = match RouteId::new(route_id.to_string()) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Cannot create RouteId \"{route_id}\": {e}");
            return false;
        }
    };
    let tid = match ResourceId::new(target.to_string()) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Cannot create ResourceId \"{target}\": {e}");
            return false;
        }
    };
    let Some(pids) = parse_ids(prereqs) else {
        return false;
    };
    match Route::new(rid, label.to_string(), tid, pids) {
        Ok(route) => match map.insert_route(route) {
            Ok(()) => true,
            Err(e) => {
                eprintln!("Cannot insert route \"{route_id}\": {e}");
                false
            }
        },
        Err(e) => {
            eprintln!("Cannot create route \"{route_id}\": {e}");
            false
        }
    }
}

/// Builds the fictional example map, printing an error on failure.
fn example_map() -> Option<DependencyMap> {
    let mut map = DependencyMap::new();

    for (id, label) in [
        ("email", "Test email"),
        ("password-store", "Test password manager"),
        ("phone", "Test phone"),
        ("recovery-sheet", "Test recovery sheet"),
    ] {
        if !insert(&mut map, id, label) {
            return None;
        }
    }

    // Two routes toward the same target: AND inside each, OR across them.
    if !insert_route(
        &mut map,
        "email-login",
        "Email login",
        "email",
        &["password-store", "phone"],
    ) || !insert_route(
        &mut map,
        "email-recovery",
        "Email recovery",
        "email",
        &["recovery-sheet"],
    ) {
        return None;
    }

    // Both known procedures for email are documented.
    let email_id = match ResourceId::new("email".to_string()) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Cannot create ResourceId \"email\": {e}");
            return None;
        }
    };
    if let Err(e) = map.set_coverage(&email_id, DocumentationCoverage::Complete) {
        eprintln!("Cannot set coverage: {e}");
        return None;
    }

    Some(map)
}

/// Prints one resource with the access status recorded by the evaluation.
///
/// The order of the checks is the presentation rule: an exclusion is imposed
/// by the scenario, proven access wins over coverage, and coverage only
/// qualifies a resource without any access evidence.
fn print_resource_status(resource: &Resource, evaluation: &Evaluation, scenario: &Scenario) {
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
    println!("{} ({}): {status}", id.as_str(), resource.label());
}

/// Prints a blocked route with the prerequisites it is missing.
fn print_blocked_route(blocked: &BlockedRoute) {
    let missing: Vec<&str> = blocked.missing().iter().map(ResourceId::as_str).collect();
    println!(
        "{} -> {}: missing {}",
        blocked.route().as_str(),
        blocked.target().as_str(),
        missing.join(", ")
    );
}

/// Evaluates one scenario on `map` and prints every resource and blocked route.
///
/// Returns `false` and prints an error when the scenario is invalid.
fn print_scenario_access(
    map: &DependencyMap,
    title: &str,
    initial: Vec<ResourceId>,
    excluded: Vec<ResourceId>,
) -> bool {
    let scenario = match Scenario::new(initial, excluded) {
        Ok(scenario) => scenario,
        Err(e) => {
            eprintln!("Cannot create scenario \"{title}\": {e}");
            return false;
        }
    };
    let evaluation = match Evaluation::evaluate(map, &scenario) {
        Ok(evaluation) => evaluation,
        Err(e) => {
            eprintln!("Cannot evaluate scenario \"{title}\": {e}");
            return false;
        }
    };

    println!("Scenario: {title}");
    for resource in map.resources() {
        print_resource_status(resource, &evaluation, &scenario);
    }
    if evaluation.blocked_routes().is_empty() {
        println!("Blocked routes: none");
    } else {
        for blocked in evaluation.blocked_routes() {
            print_blocked_route(blocked);
        }
    }
    true
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("opsecmap {VERSION}");
        return ExitCode::SUCCESS;
    }

    println!("OpSecMap v{VERSION} — fictional example map");

    let Some(map) = example_map() else {
        return ExitCode::FAILURE;
    };

    for (title, initial, excluded) in SCENARIOS {
        let (Some(initial), Some(excluded)) = (parse_ids(initial), parse_ids(excluded)) else {
            return ExitCode::FAILURE;
        };
        println!();
        if !print_scenario_access(&map, title, initial, excluded) {
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}
