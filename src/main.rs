//! Command line entry point for OpSecMap.
//!
//! This binary parses arguments and formats output; the logic lives in the
//! library crate so it stays testable without a terminal.

use opsecmap::{create_resource, DependencyMap, ResourceId, Route, RouteId, VERSION};
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

    ExitCode::SUCCESS
}
