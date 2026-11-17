// Copyright (c) 2026 onyks-os
// SPDX-License-Identifier: AGPL-3.0-or-later
//!
//! Command line entry point for OpSecMap.
//!
//! This binary handles arguments, reads commands from standard input and prints
//! their results. Parsing, execution and rendering live in the `cli` module; the
//! model and evaluator live in the library crate.

mod cli;

use std::io::{self, BufRead, IsTerminal, Write};
use std::process::ExitCode;

use cli::command::{self, Command};
use cli::session::Session;
use opsecmap::VERSION;

/// Commands that build the fictional demo map.
const DEMO_MAP: &[&str] = &[
    "resource email Test email",
    "resource password-store Test password manager",
    "resource phone Test phone",
    "resource recovery-sheet Test recovery sheet",
    // Two routes toward the same target: AND inside each, OR across them.
    "route email-login email password-store,phone Email login",
    "route email-recovery email recovery-sheet Email recovery",
    // Both known procedures for email are documented.
    "coverage email complete",
];

/// Demo scenarios: title and the `evaluate` command that runs them.
const DEMO_SCENARIOS: &[(&str, &str)] = &[
    ("Phone lost", "evaluate recovery-sheet phone"),
    ("Only password manager", "evaluate password-store -"),
];

const USAGE: &str = "\
Usage: opsecmap [--demo | --help | --version]

Without arguments, opsecmap reads commands from standard input, one per line.
Type \"help\" for the list of commands, or run: opsecmap < commands.txt

  --demo         evaluate two scenarios on a fictional example map
  -h, --help     print this message
  -V, --version  print the version";

/// Parses and executes one line, returning the lines to print.
///
/// Returns `Ok(None)` for `quit`, so the caller can stop reading.
fn run_line(session: &mut Session, line: &str) -> Result<Option<Vec<String>>, String> {
    match command::parse(line) {
        Ok(None) => Ok(Some(Vec::new())),
        Ok(Some(Command::Quit)) => Ok(None),
        Ok(Some(command)) => session
            .execute(command)
            .map(Some)
            .map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

/// Builds the demo map and evaluates each demo scenario.
fn run_demo() -> ExitCode {
    println!("OpSecMap v{VERSION} — fictional example map");

    let mut session = Session::default();
    for line in DEMO_MAP {
        if let Err(e) = run_line(&mut session, line) {
            eprintln!("error in demo map (\"{line}\"): {e}");
            return ExitCode::FAILURE;
        }
    }

    for (title, line) in DEMO_SCENARIOS {
        println!();
        println!("Scenario: {title}");
        match run_line(&mut session, line) {
            Ok(output) => {
                for output_line in output.unwrap_or_default() {
                    println!("{output_line}");
                }
            }
            Err(e) => {
                eprintln!("error in demo scenario \"{title}\": {e}");
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}

/// Reads commands from standard input until `quit` or the end of input.
///
/// A failed command is reported on stderr with its line number and the next
/// line is read. The exit code is a failure if any command failed.
fn run_session() -> ExitCode {
    let stdin = io::stdin();
    let interactive = stdin.is_terminal();
    if interactive {
        println!("OpSecMap v{VERSION}. Type \"help\" for the list of commands, \"quit\" to exit.");
    }

    let mut session = Session::default();
    let mut input = stdin.lock();
    let mut line = String::new();
    let mut line_number = 0_usize;
    let mut failures = 0_usize;
    loop {
        if interactive {
            print!("> ");
            // A failed flush only delays the prompt; reading still works.
            let _ = io::stdout().flush();
        }
        line.clear();
        match input.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => {
                eprintln!("error: cannot read input: {e}");
                return ExitCode::FAILURE;
            }
        }
        line_number += 1;

        match run_line(&mut session, &line) {
            Ok(Some(output)) => {
                for output_line in output {
                    println!("{output_line}");
                }
            }
            Ok(None) => break,
            Err(e) => {
                eprintln!("error (line {line_number}): {e}");
                failures += 1;
            }
        }
    }

    if failures == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] => run_session(),
        ["--demo"] => run_demo(),
        ["--version" | "-V"] => {
            println!("opsecmap {VERSION}");
            ExitCode::SUCCESS
        }
        ["--help" | "-h"] => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}
