//! Command line entry point for OpSecMap.
//!
//! This binary parses arguments and formats output; the logic lives in the
//! library crate so it stays testable without a terminal.

use std::process::ExitCode;

use opsecmap::VERSION;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("opsecmap {VERSION}");
        return ExitCode::SUCCESS;
    }

    // TODO(template): dispatch to the library here.
    println!("OpSecMap v{VERSION}");
    ExitCode::SUCCESS
}
