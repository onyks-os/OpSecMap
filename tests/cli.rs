// Copyright (c) 2026 onyks-os
// SPDX-License-Identifier: AGPL-3.0-or-later
//!
//! Integration tests that run the `opsecmap` binary with arguments and stdin.

#![allow(clippy::expect_used)]

use std::io::Write;
use std::process::{Command, Output, Stdio};

const FIXTURE: &str = include_str!("fixtures/phone-lost.txt");

/// Runs the binary with `args`, writing `input` to its standard input.
fn run(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_opsecmap"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("binary must start");
    child
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(input.as_bytes())
        .expect("stdin must accept input");
    child.wait_with_output().expect("binary must finish")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout is UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr is UTF-8")
}

#[test]
fn fixture_prints_the_same_report_lines_as_the_demo() {
    let session = run(&[], FIXTURE);
    let demo = run(&["--demo"], "");

    assert!(session.status.success(), "{}", stderr(&session));
    assert!(demo.status.success(), "{}", stderr(&demo));
    assert_eq!(stderr(&session), "");

    // The demo adds a banner, blank lines and scenario titles around the same reports.
    let demo_report: Vec<String> = stdout(&demo)
        .lines()
        .skip(1)
        .filter(|line| !line.is_empty() && !line.starts_with("Scenario: "))
        .map(str::to_string)
        .collect();
    let session_report: Vec<String> = stdout(&session).lines().map(str::to_string).collect();
    assert_eq!(session_report, demo_report);
    assert_eq!(
        session_report.first().map(String::as_str),
        Some("Initial: recovery-sheet; excluded: phone")
    );
}

#[test]
fn failed_line_is_reported_and_later_lines_still_run() {
    let output = run(
        &[],
        "resource phone Test phone\nresource phone Duplicate\nfrobnicate\nlist\n",
    );

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stderr(&output),
        "error (line 2): resource \"phone\" already exists\n\
         error (line 3): unknown command \"frobnicate\"; type \"help\" for the list\n"
    );
    assert_eq!(stdout(&output), "resource phone Test phone\n");
}

#[test]
fn quit_stops_reading() {
    let output = run(&[], "resource phone Test phone\nquit\nfrobnicate\n");

    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(stderr(&output), "");
}

#[test]
fn empty_input_succeeds_without_output() {
    let output = run(&[], "");

    assert!(output.status.success());
    assert_eq!(stdout(&output), "");
}

#[test]
fn version_and_help_flags() {
    let version = run(&["--version"], "");
    assert!(version.status.success());
    assert_eq!(
        stdout(&version),
        format!("opsecmap {}\n", env!("CARGO_PKG_VERSION"))
    );

    let help = run(&["--help"], "");
    assert!(help.status.success());
    assert!(stdout(&help).starts_with("Usage: opsecmap"));
}

#[test]
fn unknown_argument_prints_usage_and_exits_with_2() {
    let output = run(&["--frobnicate"], "");

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).starts_with("Usage: opsecmap"));
    assert_eq!(stdout(&output), "");
}
