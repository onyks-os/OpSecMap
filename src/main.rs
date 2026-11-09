// Copyright (c) 2026 onyks-it
// SPDX-License-Identifier: AGPL-3.0-or-later
//!
//! Command line entry point for OpSecMap.
//!
//! This binary parses arguments and formats output; the logic lives in the
//! library crate so it stays testable without a terminal.

use opsecmap::{
    create_resource, AccessEvidence, AccessOrigin, DependencyMap, Evaluation, EvaluationError,
    ResourceId, Route, RouteId, Scenario, VERSION,
};
use std::process::ExitCode;

/// Inserts a resource built from raw strings, printing an error and returning
/// `false` on failure.
fn insert(map: &mut DependencyMap, id: &str, label: &str) -> bool {
    match create_resource(id.to_string(), label.to_string()) {
        Ok(resource) => match map.insert(resource) {
            Ok(()) => true,
            Err(e) => {
                eprintln!("Cannot insert resource \"{id}\": {e:?}");
                false
            }
        },
        Err(e) => {
            eprintln!("Cannot create resource \"{id}\": {e:?}");
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
            eprintln!("Cannot create RouteId \"{route_id}\": {e:?}");
            return false;
        }
    };
    let tid = match ResourceId::new(target.to_string()) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Cannot create ResourceId \"{target}\": {e:?}");
            return false;
        }
    };
    let pids: Vec<ResourceId> = prereqs
        .iter()
        .filter_map(|p| match ResourceId::new((*p).to_string()) {
            Ok(id) => Some(id),
            Err(e) => {
                eprintln!("Cannot create ResourceId \"{p}\": {e:?}");
                None
            }
        })
        .collect();
    if pids.len() != prereqs.len() {
        return false;
    }
    match Route::new(rid, label.to_string(), tid, pids) {
        Ok(route) => match map.insert_route(route) {
            Ok(()) => true,
            Err(e) => {
                eprintln!("Cannot insert route \"{route_id}\": {e:?}");
                false
            }
        },
        Err(e) => {
            eprintln!("Cannot create route \"{route_id}\": {e:?}");
            false
        }
    }
}

/// Prints the resource ID and the origin recorded by an evaluation.
fn print_evidence(evidence: &AccessEvidence) {
    match evidence.origin() {
        AccessOrigin::Initial => println!("{}: initially accessible", evidence.resource().as_str()),
        AccessOrigin::Derived { route } => println!(
            "{}: accessible through {}",
            evidence.resource().as_str(),
            route.as_str()
        ),
    }
}

/// Evaluates and prints access for a scenario on the example map.
fn print_scenario_access(
    map: &DependencyMap,
    phone_id: ResourceId,
    email_id: &ResourceId,
) -> ExitCode {
    // Phone starts accessible and email is excluded.
    let Ok(scenario) = Scenario::new(vec![phone_id], vec![email_id.clone()]) else {
        eprintln!("Cannot create example scenario");
        return ExitCode::FAILURE;
    };
    let evaluation = match Evaluation::evaluate(map, &scenario) {
        Ok(evaluation) => evaluation,
        Err(EvaluationError::UnknownInitialResource(id)) => {
            eprintln!("Unknown initial resource: {}", id.as_str());
            return ExitCode::FAILURE;
        }
        Err(EvaluationError::UnknownExcludedResource(id)) => {
            eprintln!("Unknown excluded resource: {}", id.as_str());
            return ExitCode::FAILURE;
        }
    };

    println!("Scenario access:");
    for resource in map.resources() {
        if let Some(evidence) = evaluation.evidence_for(resource.id()) {
            print_evidence(evidence);
        }
    }
    println!("Email accessible: {}", evaluation.is_accessible(email_id));

    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("opsecmap {VERSION}");
        return ExitCode::SUCCESS;
    }

    println!("OpSecMap v{VERSION}");

    let mut map = DependencyMap::new();

    // Four example resources.
    for (id, label) in [
        ("email", "Test email"),
        ("password-store", "Test password manager"),
        ("phone", "Test phone"),
        ("recovery-sheet", "Test recovery sheet"),
    ] {
        if !insert(&mut map, id, label) {
            return ExitCode::FAILURE;
        }
    }

    // Two routes toward the same target: AND inside each, OR across them.
    if !insert_route(
        &mut map,
        "email-login",
        "Email login",
        "email",
        &["password-store", "phone"],
    ) {
        return ExitCode::FAILURE;
    }
    if !insert_route(
        &mut map,
        "email-recovery",
        "Email recovery",
        "email",
        &["recovery-sheet"],
    ) {
        return ExitCode::FAILURE;
    }

    // Rename phone after routes have been registered.
    let phone_id = match ResourceId::new("phone".to_string()) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Cannot create ResourceId: {e:?}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = map.rename(&phone_id, "Backup phone".to_string()) {
        eprintln!("Cannot rename resource: {e:?}");
        return ExitCode::FAILURE;
    }

    // Verify email-login still references the stable phone identifier.
    let email_login_id = match RouteId::new("email-login".to_string()) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Cannot create RouteId: {e:?}");
            return ExitCode::FAILURE;
        }
    };
    if let Some(route) = map.find_route(&email_login_id) {
        if !route.prerequisites().contains(&phone_id) {
            eprintln!("Integrity error: email-login lost phone prerequisite");
            return ExitCode::FAILURE;
        }
    } else {
        eprintln!("Route email-login not found");
        return ExitCode::FAILURE;
    }

    // Summary counts.
    println!("Resources: {}", map.resources().len());
    println!("Routes:    {}", map.routes().len());

    // Print each route with its target and prerequisite group.
    for route in map.routes() {
        println!(
            "{} — target: {}",
            route.id().as_str(),
            route.target().as_str()
        );
        let prereqs: Vec<&str> = route
            .prerequisites()
            .iter()
            .map(ResourceId::as_str)
            .collect();
        println!("  requires all: {}", prereqs.join(", "));
    }

    // Count and list distinct alternative routes for "email".
    let email_id = match ResourceId::new("email".to_string()) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Cannot create ResourceId: {e:?}");
            return ExitCode::FAILURE;
        }
    };
    let alternatives = map
        .routes()
        .iter()
        .filter(|r| r.target() == &email_id)
        .count();
    println!("Alternatives for email: {alternatives}");

    print_scenario_access(&map, phone_id, &email_id)
}
