//! Command line entry point for OpSecMap.
//!
//! This binary parses arguments and formats output; the logic lives in the
//! library crate so it stays testable without a terminal.

use opsecmap::{create_resource, DependencyMap, Resource, ResourceId, Route, RouteId, VERSION};
use std::process::ExitCode;

/// Formats a resource for terminal output.
#[must_use]
fn describe_resource(r: &Resource) -> String {
    format!("Id: {} - Name: {}", r.id().as_str(), r.label())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("opsecmap {VERSION}");
        return ExitCode::SUCCESS;
    }

    println!("OpSecMap v{VERSION}");

    let phone = match create_resource("id1".to_string(), "Pixel 10".to_string()) {
        Ok(resource) => resource,
        Err(error) => {
            eprintln!("Cannot create resource: {error:?}");
            return ExitCode::FAILURE;
        }
    };

    let account = match create_resource("id2".to_string(), "name.surname@proton.me".to_string()) {
        Ok(resource) => resource,
        Err(error) => {
            eprintln!("Cannot create resource: {error:?}");
            return ExitCode::FAILURE;
        }
    };

    let recovery = match create_resource("id3".to_string(), "Recovery sheet".to_string()) {
        Ok(resource) => resource,
        Err(error) => {
            eprintln!("Cannot create resource: {error:?}");
            return ExitCode::FAILURE;
        }
    };

    let pwdman = match create_resource("id4".to_string(), "Bitwarden".to_string()) {
        Ok(resource) => resource,
        Err(error) => {
            eprintln!("Cannot create resource: {error:?}");
            return ExitCode::FAILURE;
        }
    };

    let account_id = account.id().clone();
    let mut dependency_map = DependencyMap::new();
    for resource in [phone, account, recovery, pwdman] {
        match dependency_map.insert(resource) {
            Ok(()) => {}
            Err(error) => {
                eprintln!("Cannot insert resource: {error:?}");
                return ExitCode::FAILURE;
            }
        }
    }

    let phone_id = match ResourceId::new("id1".to_string()) {
        Ok(id) => id,
        Err(error) => {
            eprintln!("Cannot create ID {error:?}");
            return ExitCode::FAILURE;
        }
    };

    match dependency_map.rename(&phone_id, "Samsung Galaxy".to_string()) {
        Ok(()) => println!("Label changed successfully"),
        Err(error) => {
            eprintln!("Cannot change label {error:?}");
        }
    }

    for resource in dependency_map.resources() {
        println!("{}", describe_resource(resource));
    }

    let route_id = match RouteId::new("email-login".to_string()) {
        Ok(id) => id,
        Err(error) => {
            eprintln!("Cannot create RouteId: {error:?}");
            return ExitCode::FAILURE;
        }
    };
    println!("Route ID: {}", route_id.as_str());

    let pwdman_id = match ResourceId::new("id4".to_string()) {
        Ok(id) => id,
        Err(error) => {
            eprintln!("Cannot create ResourceId {error:?}");
            return ExitCode::FAILURE;
        }
    };
    let prerequisites = vec![pwdman_id, phone_id.clone()];
    let route = match Route::new(
        route_id,
        "Email login".to_string(),
        account_id,
        prerequisites,
    ) {
        Ok(route) => route,
        Err(error) => {
            eprintln!("Cannot create Route {error:?}");
            return ExitCode::FAILURE;
        }
    };
    for prerequisite in route.prerequisites() {
        println!("Prerequisite: {}", prerequisite.as_str());
    }
    ExitCode::SUCCESS
}
