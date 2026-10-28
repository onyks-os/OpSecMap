//! Command line entry point for OpSecMap.
//!
//! This binary parses arguments and formats output; the logic lives in the
//! library crate so it stays testable without a terminal.

use opsecmap::{create_resource, DependencyMap, Resource, ResourceId, VERSION};
use std::process::ExitCode;

/// Describes the resource
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

    // TODO(template): dispatch to the library here.
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
            eprintln!("Cannot craete resource: {error:?}");
            return ExitCode::FAILURE;
        }
    };

    let mut inventory = DependencyMap::new();

    for resource in [phone, account, recovery, pwdman] {
        match inventory.insert(resource) {
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

    match inventory.rename(&phone_id, "Samsung Galaxy".to_string()) {
        Ok(()) => println!("Label changed successfully"),
        Err(error) => {
            eprintln!("Cannot change label {error:?}");
        }
    }

    for i in inventory.resources() {
        println!("{}", describe_resource(i));
    }

    ExitCode::SUCCESS
}
