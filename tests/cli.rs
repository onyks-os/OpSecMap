// Copyright (c) 2026 onyks-os
// SPDX-License-Identifier: AGPL-3.0-or-later
//!
//! Integration tests that run the `opsecmap` binary with arguments and stdin.

#![allow(clippy::expect_used)]

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

const FIXTURE: &str = include_str!("fixtures/phone-lost.txt");

/// The variable that supplies the password instead of the terminal.
const PASSWORD_ENV: &str = "OPSECMAP_PASSWORD";

/// A fictional password for the tests.
const PASSWORD: &str = "test password";

/// Runs the binary with `args`, writing `input` to its standard input.
///
/// [`PASSWORD_ENV`] is removed, so a command that asks for a password reads the
/// terminal: without one, as in CI, it fails with an error instead of waiting.
fn run(args: &[&str], input: &str) -> Output {
    run_with(args, input, None)
}

/// Like [`run`], with [`PASSWORD_ENV`] set to `password`.
fn run_with_password(input: &str, password: &str) -> Output {
    run_with(&[], input, Some(password))
}

fn run_with(args: &[&str], input: &str, password: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_opsecmap"));
    command.env_remove(PASSWORD_ENV);
    if let Some(password) = password {
        command.env(PASSWORD_ENV, password);
    }
    let mut child = command
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

    // The demo adds a banner, export and open confirmations, blank lines and
    // scenario titles around the same reports. It exports, so it never asks for
    // a password.
    let demo_out = stdout(&demo);
    let mut demo_lines = demo_out.lines().skip(1);
    assert!(demo_lines.next().is_some_and(|line| line
        .starts_with("exported 4 resources and 2 routes to ")
        && line.ends_with(" (plain text, NOT ENCRYPTED)")));
    assert!(demo_lines.next().is_some_and(|line| line
        .starts_with("opened 4 resources and 2 routes from ")
        && line.ends_with(" (plain text)")));
    let demo_report: Vec<String> = demo_lines
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

/// A folder of its own for one test, removed when the test ends, even if it fails.
struct TempDir(PathBuf);

impl TempDir {
    /// Creates an empty folder named after the process and `test`, so parallel
    /// tests and concurrent runs never share files.
    fn new(test: &str) -> Self {
        let path = std::env::temp_dir().join(format!("opsecmap-{}-{test}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("temporary folder must be created");
        Self(path)
    }

    /// The path of `name` inside the folder, as text for a command line.
    fn file(&self, name: &str) -> String {
        self.0.join(name).display().to_string()
    }

    /// The names of the files in the folder, sorted.
    fn entries(&self) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(&self.0)
            .expect("temporary folder must be readable")
            .map(|entry| {
                entry
                    .expect("entry must be readable")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        names.sort();
        names
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The commands of the fixture that build the map, without its evaluations.
fn fixture_map_commands() -> String {
    FIXTURE
        .lines()
        .filter(|line| !line.starts_with("evaluate"))
        .flat_map(|line| [line, "\n"])
        .collect()
}

const MAP_QUERIES: &str = "list\nevaluate recovery-sheet phone\nevaluate password-store -\n";

#[test]
fn saved_map_reopens_with_the_same_list_and_evaluations() {
    let dir = TempDir::new("reopen");
    let map = dir.file("map.opsecmap");

    let first = run_with_password(
        &format!("{}{MAP_QUERIES}save {map}\n", fixture_map_commands()),
        PASSWORD,
    );
    assert!(first.status.success(), "{}", stderr(&first));
    let first_out = stdout(&first);
    let report = first_out
        .strip_suffix(&format!(
            "saved 4 resources and 2 routes to {map} (encrypted)\n"
        ))
        .expect("save must confirm on the last line");

    // A resource added before `open` is replaced, not merged.
    let second = run_with_password(
        &format!("resource extra Test extra\nopen {map}\n{MAP_QUERIES}"),
        PASSWORD,
    );
    assert!(second.status.success(), "{}", stderr(&second));
    assert_eq!(
        stdout(&second),
        format!("opened 4 resources and 2 routes from {map} (encrypted)\n{report}")
    );
}

#[test]
fn saved_file_is_encrypted_and_holds_no_map_text() {
    let dir = TempDir::new("ciphertext");
    let map = dir.file("map.opsecmap");

    let output = run_with_password(&format!("{}save {map}\n", fixture_map_commands()), PASSWORD);
    assert!(output.status.success(), "{}", stderr(&output));

    let bytes = fs::read(&map).expect("saved file must exist");
    assert!(bytes.starts_with(b"OPSECMAP"));
    let contains = |needle: &[u8]| bytes.windows(needle.len()).any(|window| window == needle);
    for text in ["phone", "recovery-sheet", "Test", "resources", PASSWORD] {
        assert!(!contains(text.as_bytes()), "{text}");
    }
}

#[test]
fn wrong_password_or_changed_byte_leaves_the_previous_map() {
    let dir = TempDir::new("wrong-password");
    let map = dir.file("map.opsecmap");
    let changed = dir.file("changed.opsecmap");
    let truncated = dir.file("truncated.opsecmap");
    let saved = run_with_password(
        &format!("resource phone Test phone\nsave {map}\n"),
        PASSWORD,
    );
    assert!(saved.status.success(), "{}", stderr(&saved));
    let mut bytes = fs::read(&map).expect("saved file must exist");
    fs::write(&truncated, &bytes[..60]).expect("fixture must be written");
    let last = bytes.len() - 1;
    bytes[last] ^= 1;
    fs::write(&changed, &bytes).expect("fixture must be written");

    let wrong = run_with_password(
        &format!("resource email Test email\nopen {map}\nlist\n"),
        "wrong password",
    );
    let damaged = run_with_password(
        &format!("resource email Test email\nopen {changed}\nopen {truncated}\nlist\n"),
        PASSWORD,
    );

    let message = "wrong password, or the map file is damaged or was modified";
    assert_eq!(wrong.status.code(), Some(1));
    assert_eq!(stderr(&wrong), format!("error (line 2): {message}\n"));
    assert_eq!(stdout(&wrong), "resource email Test email\n");
    assert_eq!(damaged.status.code(), Some(1));
    assert_eq!(
        stderr(&damaged),
        format!(
            "error (line 2): {message}\n\
             error (line 3): encrypted map file is truncated\n"
        )
    );
    assert_eq!(stdout(&damaged), "resource email Test email\n");
}

#[test]
fn exported_map_is_plain_json_and_reopens_without_a_password() {
    let dir = TempDir::new("export");
    let map = dir.file("map.json");

    let first = run(
        &[],
        &format!("{}{MAP_QUERIES}export {map}\n", fixture_map_commands()),
    );
    assert!(first.status.success(), "{}", stderr(&first));
    let first_out = stdout(&first);
    let report = first_out
        .strip_suffix(&format!(
            "exported 4 resources and 2 routes to {map} (plain text, NOT ENCRYPTED)\n"
        ))
        .expect("export must confirm on the last line");
    let text = fs::read_to_string(&map).expect("exported file must be text");
    assert!(text.starts_with('{') && text.contains("\"recovery-sheet\""));

    // No password variable is set: a plain file must not ask for one.
    let second = run(&[], &format!("open {map}\n{MAP_QUERIES}"));
    assert!(second.status.success(), "{}", stderr(&second));
    assert_eq!(
        stdout(&second),
        format!("opened 4 resources and 2 routes from {map} (plain text)\n{report}")
    );
}

#[test]
fn failed_open_leaves_the_previous_map() {
    let dir = TempDir::new("failed-open");
    let missing = dir.file("missing.json");
    let invalid = dir.file("invalid.json");
    let future = dir.file("future.json");
    fs::write(&invalid, "{ not json").expect("fixture must be written");
    fs::write(
        &future,
        r#"{"format": "opsecmap-map", "version": 2, "resources": [], "routes": []}"#,
    )
    .expect("fixture must be written");

    let output = run(
        &[],
        &format!(
            "resource phone Test phone\nopen {missing}\nopen {invalid}\nopen {future}\nlist\n"
        ),
    );

    assert_eq!(output.status.code(), Some(1));
    let errors = stderr(&output);
    let errors: Vec<&str> = errors.lines().collect();
    assert_eq!(errors.len(), 3, "{errors:?}");
    assert!(errors[0].starts_with("error (line 2): cannot access the map file: "));
    assert!(errors[1].starts_with("error (line 3): invalid map file: "));
    assert_eq!(
        errors[2],
        "error (line 4): unsupported map file version 2 (this build reads version 1)"
    );
    assert_eq!(stdout(&output), "resource phone Test phone\n");
}

#[test]
fn file_over_the_size_limit_is_refused_before_the_password() {
    const MAX_FILE_BYTES: usize = 1_048_576;
    let dir = TempDir::new("size-limit");
    let at_limit = dir.file("at-limit.json");
    let over_limit = dir.file("over-limit.json");
    let encrypted_over_limit = dir.file("over-limit.opsecmap");
    fs::write(&at_limit, " ".repeat(MAX_FILE_BYTES)).expect("fixture must be written");
    fs::write(&over_limit, " ".repeat(MAX_FILE_BYTES + 1)).expect("fixture must be written");
    let mut encrypted = b"OPSECMAP".to_vec();
    encrypted.resize(MAX_FILE_BYTES + 1, 0);
    fs::write(&encrypted_over_limit, encrypted).expect("fixture must be written");

    // No password variable is set: asking for one would fail with a terminal
    // error, so the size error proves that the password was not asked.
    let output = run(
        &[],
        &format!("open {at_limit}\nopen {over_limit}\nopen {encrypted_over_limit}\n"),
    );

    // A file exactly at the limit is read, and fails only as JSON.
    let errors = stderr(&output);
    let errors: Vec<&str> = errors.lines().collect();
    assert_eq!(errors.len(), 3, "{errors:?}");
    assert!(errors[0].starts_with("error (line 1): invalid map file: "));
    assert_eq!(
        errors[1],
        "error (line 2): map file is larger than 1048576 bytes"
    );
    assert_eq!(
        errors[2],
        "error (line 3): map file is larger than 1048576 bytes"
    );
}

#[test]
fn save_leaves_no_temporary_file_and_overwrites() {
    let dir = TempDir::new("no-tmp");
    let map = dir.file("map.opsecmap");

    let output = run_with_password(
        &format!("resource phone Test phone\nsave {map}\nresource email Test email\nsave {map}\n"),
        PASSWORD,
    );

    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(dir.entries(), ["map.opsecmap"]);
    let reopened = run_with_password(&format!("open {map}\nlist\n"), PASSWORD);
    assert!(stdout(&reopened).ends_with("resource phone Test phone\nresource email Test email\n"));
}

#[test]
fn failed_save_is_reported_before_the_password_and_leaves_no_file() {
    let dir = TempDir::new("failed-save");
    let map = dir.file("missing-folder/map.opsecmap");

    // No password variable is set: the storage error proves that the password
    // was not asked for a destination that cannot be written.
    let output = run(
        &[],
        &format!("resource phone Test phone\nsave {map}\nexport {map}\n"),
    );

    assert_eq!(output.status.code(), Some(1));
    let errors = stderr(&output);
    let errors: Vec<&str> = errors.lines().collect();
    assert_eq!(errors.len(), 2, "{errors:?}");
    assert!(errors[0].starts_with("error (line 2): cannot access the map file: "));
    assert!(errors[1].starts_with("error (line 3): cannot access the map file: "));
    assert_eq!(dir.entries(), Vec::<String>::new());
}

#[test]
fn empty_password_is_refused_and_writes_nothing() {
    let dir = TempDir::new("empty-password");
    let map = dir.file("map.opsecmap");

    let output = run_with_password(&format!("resource phone Test phone\nsave {map}\n"), "");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stderr(&output), "error (line 2): the password is empty\n");
    assert_eq!(dir.entries(), Vec::<String>::new());
}

#[test]
fn path_with_a_space_works_for_save_export_and_open() {
    let dir = TempDir::new("space");
    let saved = dir.file("my map.opsecmap");
    let exported = dir.file("my map.json");

    let output = run_with_password(
        &format!("resource phone Test phone\nsave {saved}\nexport {exported}\n"),
        PASSWORD,
    );
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(dir.entries(), ["my map.json", "my map.opsecmap"]);

    let opened = run_with_password(&format!("open {saved}\nopen {exported}\nlist\n"), PASSWORD);
    assert!(opened.status.success(), "{}", stderr(&opened));
    assert!(stdout(&opened).ends_with("resource phone Test phone\n"));
}

#[test]
fn save_export_and_open_require_a_path() {
    let output = run(&[], "save\nexport \nopen   \n");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        stderr(&output),
        "error (line 1): missing argument: path\n\
         error (line 2): missing argument: path\n\
         error (line 3): missing argument: path\n"
    );
}
