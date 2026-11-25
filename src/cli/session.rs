// Copyright (c) 2026 onyks-os
// SPDX-License-Identifier: AGPL-3.0-or-later
//!
//! A map being edited from the command line, and the execution of commands on it.

use std::fmt;

use opsecmap::{
    decrypt, encrypt, from_json, is_encrypted, to_json, CryptoError, DependencyMap,
    DocumentationCoverage, Evaluation, EvaluationError, MapError, MapFileError, Resource,
    ResourceError, ResourceId, Route, RouteError, Scenario, ScenarioError,
};
use zeroize::Zeroizing;

use crate::cli::password::{read_new_password, read_password, PasswordError};
use crate::cli::storage::{into_text, StorageError};

use super::command::{coverage_keyword, Command, HELP};
use super::render;
use super::storage::{check_writable, read_map_file, write_map_file};

/// The map built by the commands of one run.
#[derive(Debug, Default)]
pub(crate) struct Session {
    /// The map every command reads or changes.
    map: DependencyMap,
}

/// An error raised while executing a command.
///
/// Each variant wraps the error of one library or file operation; the `From` impls
/// below let `?` convert them without an explicit `match`.
#[derive(Debug)]
pub(crate) enum SessionError {
    /// A resource could not be constructed.
    Resource(ResourceError),
    /// A route could not be constructed.
    Route(RouteError),
    /// The map rejected an insertion, update or removal.
    Map(MapError),
    /// The scenario is inconsistent.
    Scenario(ScenarioError),
    /// The scenario refers to resources absent from the map.
    Evaluation(EvaluationError),
    /// The map file could not be read or written, or is too large.
    Storage(StorageError),
    /// The map file was read but is not a valid map.
    File(MapFileError),
    /// The password could not be read, was empty, or was not repeated exactly.
    Password(PasswordError),
    /// The map file could not be encrypted or decrypted.
    Crypto(CryptoError),
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Resource(e) => write!(f, "{e}"),
            Self::Route(e) => write!(f, "{e}"),
            Self::Map(e) => write!(f, "{e}"),
            Self::Scenario(e) => write!(f, "{e}"),
            Self::Evaluation(e) => write!(f, "{e}"),
            Self::Storage(e) => write!(f, "{e}"),
            Self::File(e) => write!(f, "{e}"),
            Self::Password(e) => write!(f, "{e}"),
            Self::Crypto(e) => write!(f, "{e}"),
        }
    }
}

impl From<ResourceError> for SessionError {
    fn from(e: ResourceError) -> Self {
        Self::Resource(e)
    }
}

impl From<RouteError> for SessionError {
    fn from(e: RouteError) -> Self {
        Self::Route(e)
    }
}

impl From<MapError> for SessionError {
    fn from(e: MapError) -> Self {
        Self::Map(e)
    }
}

impl From<ScenarioError> for SessionError {
    fn from(e: ScenarioError) -> Self {
        Self::Scenario(e)
    }
}

impl From<EvaluationError> for SessionError {
    fn from(e: EvaluationError) -> Self {
        Self::Evaluation(e)
    }
}

impl From<StorageError> for SessionError {
    fn from(e: StorageError) -> Self {
        Self::Storage(e)
    }
}

impl From<MapFileError> for SessionError {
    fn from(e: MapFileError) -> Self {
        Self::File(e)
    }
}

impl From<PasswordError> for SessionError {
    fn from(e: PasswordError) -> Self {
        Self::Password(e)
    }
}

impl From<CryptoError> for SessionError {
    fn from(e: CryptoError) -> Self {
        Self::Crypto(e)
    }
}

impl Session {
    /// Applies `command` to the map and returns the lines to print.
    ///
    /// `Evaluate` returns a header with the scenario's assumptions followed by
    /// the report from [`render::scenario_report`].
    ///
    /// A failed command leaves the map unchanged: every library operation it
    /// calls either succeeds completely or changes nothing. `Quit` is handled by
    /// the caller and does nothing here.
    pub(crate) fn execute(&mut self, command: Command) -> Result<Vec<String>, SessionError> {
        match command {
            Command::AddResource { id, label } => {
                self.map.insert(Resource::new(id, label)?)?;
                Ok(Vec::new())
            }
            Command::AddRoute {
                id,
                target,
                prerequisites,
                label,
            } => {
                self.map
                    .insert_route(Route::new(id, label, target, prerequisites)?)?;
                Ok(Vec::new())
            }
            Command::SetCoverage { id, coverage } => {
                self.map.set_coverage(&id, coverage)?;
                Ok(Vec::new())
            }
            Command::Evaluate { initial, excluded } => {
                let scenario = Scenario::new(initial, excluded)?;
                let evaluation = Evaluation::evaluate(&self.map, &scenario)?;
                let mut lines = vec![render::scenario_header(&scenario)];
                lines.extend(render::scenario_report(&self.map, &evaluation, &scenario));
                Ok(lines)
            }
            Command::RemoveResource { id } => {
                self.map.remove_resource(&id)?;
                Ok(Vec::new())
            }
            Command::RemoveRoute { id } => {
                self.map.remove_route(&id)?;
                Ok(Vec::new())
            }
            Command::Export { path } => {
                let text = to_json(&self.map)?;
                write_map_file(&path, text.as_bytes())?;
                Ok(vec![format!(
                    "exported {} resources and {} routes to {} (plain text, NOT ENCRYPTED)",
                    self.map.resources().len(),
                    self.map.routes().len(),
                    path.display()
                )])
            }
            Command::Save { path } => {
                let text = Zeroizing::new(to_json(&self.map)?);
                // A destination that cannot be written fails before the password is typed.
                check_writable(&path)?;
                let new_password = read_new_password()?;
                let ciphertext = encrypt(text.as_bytes(), new_password.as_bytes())?;
                write_map_file(&path, &ciphertext)?;
                Ok(vec![format!(
                    "saved {} resources and {} routes to {} (encrypted)",
                    self.map.resources().len(),
                    self.map.routes().len(),
                    path.display()
                )])
            }
            Command::Open { path } => {
                let file = read_map_file(&path)?;
                let encrypted = is_encrypted(&file);
                let pt = if encrypted {
                    let password = read_password("Password: ")?;
                    decrypt(&file, password.as_bytes())?
                } else {
                    Zeroizing::new(file)
                };
                let plaintext = into_text(&pt)?;
                let map = from_json(plaintext)?;
                self.map = map;
                Ok(vec![format!(
                    "opened {} resources and {} routes from {} ({})",
                    self.map.resources().len(),
                    self.map.routes().len(),
                    path.display(),
                    if encrypted { "encrypted" } else { "plain text" }
                )])
            }
            Command::List => Ok(self.list()),
            Command::Help => Ok(HELP.iter().map(|line| (*line).to_string()).collect()),
            Command::Quit => Ok(Vec::new()),
        }
    }

    /// Prints the map as commands that rebuild it: resources, then coverage
    /// other than `Unknown`, then routes, each in map order.
    fn list(&self) -> Vec<String> {
        let resources = self.map.resources();
        let mut lines: Vec<String> = resources
            .iter()
            .map(|resource| format!("resource {} {}", resource.id().as_str(), resource.label()))
            .collect();
        lines.extend(
            resources
                .iter()
                .filter(|resource| resource.coverage() != DocumentationCoverage::Unknown)
                .map(|resource| {
                    format!(
                        "coverage {} {}",
                        resource.id().as_str(),
                        coverage_keyword(resource.coverage())
                    )
                }),
        );
        lines.extend(self.map.routes().iter().map(|route| {
            let prerequisites: Vec<&str> = route
                .prerequisites()
                .iter()
                .map(ResourceId::as_str)
                .collect();
            format!(
                "route {} {} {} {}",
                route.id().as_str(),
                route.target().as_str(),
                prerequisites.join(","),
                route.label()
            )
        }));
        lines
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{Session, SessionError};
    use crate::cli::command::parse;
    use opsecmap::MapError;

    /// Parses and executes each line, failing the test on any error.
    fn run(session: &mut Session, lines: &[&str]) -> Vec<String> {
        let mut output = Vec::new();
        for line in lines {
            if let Some(command) = parse(line).expect("test line must parse") {
                output.extend(session.execute(command).expect("test command must succeed"));
            }
        }
        output
    }

    fn execute(session: &mut Session, line: &str) -> Result<Vec<String>, SessionError> {
        let command = parse(line)
            .expect("test line must parse")
            .expect("test line must contain a command");
        session.execute(command)
    }

    const EXAMPLE: &[&str] = &[
        "resource email Test email",
        "resource password-store Test password manager",
        "resource phone Test phone",
        "resource recovery-sheet Test recovery sheet",
        "coverage email complete",
        "route email-login email password-store,phone Email login",
        "route email-recovery email recovery-sheet Email recovery",
    ];

    #[test]
    fn commands_build_a_map_that_evaluate_reports_on() {
        let mut session = Session::default();
        run(&mut session, EXAMPLE);

        let report = run(&mut session, &["evaluate recovery-sheet phone"]);

        assert_eq!(
            report,
            [
                "Initial: recovery-sheet; excluded: phone",
                "email (Test email): accessible via email-recovery",
                "password-store (Test password manager): undetermined: documentation incomplete or not declared",
                "phone (Test phone): excluded by the scenario",
                "recovery-sheet (Test recovery sheet): available at start",
                "Blocked routes: none",
            ]
        );
    }

    #[test]
    fn list_prints_resources_then_coverage_then_routes() {
        let mut session = Session::default();
        run(&mut session, EXAMPLE);

        let listed = run(&mut session, &["list"]);

        assert_eq!(
            listed,
            [
                "resource email Test email",
                "resource password-store Test password manager",
                "resource phone Test phone",
                "resource recovery-sheet Test recovery sheet",
                "coverage email complete",
                "route email-login email password-store,phone Email login",
                "route email-recovery email recovery-sheet Email recovery",
            ]
        );
    }

    #[test]
    fn list_output_rebuilds_the_same_map() {
        let mut original = Session::default();
        run(&mut original, EXAMPLE);
        let listed = run(&mut original, &["list"]);

        let mut rebuilt = Session::default();
        let lines: Vec<&str> = listed.iter().map(String::as_str).collect();
        run(&mut rebuilt, &lines);

        assert_eq!(run(&mut rebuilt, &["list"]), listed);
    }

    #[test]
    fn failed_commands_leave_the_map_unchanged() {
        let mut session = Session::default();
        run(&mut session, EXAMPLE);
        let before = run(&mut session, &["list"]);

        for line in [
            "resource phone Another phone",
            "route email-login email phone Duplicate",
            "route vault-open vault phone Missing target",
            "route vault-open email vault Missing prerequisite",
            "coverage vault complete",
            "remove resource phone",
            "remove route missing",
        ] {
            assert!(execute(&mut session, line).is_err(), "{line}");
        }

        assert_eq!(run(&mut session, &["list"]), before);
    }

    #[test]
    fn errors_come_from_the_library_operation_that_failed() {
        let mut session = Session::default();
        run(&mut session, EXAMPLE);

        assert!(matches!(
            execute(&mut session, "remove resource phone"),
            Err(SessionError::Map(MapError::ResourceInUse { .. }))
        ));
        assert!(matches!(
            execute(&mut session, "evaluate vault -"),
            Err(SessionError::Evaluation(_))
        ));
        assert!(matches!(
            execute(&mut session, "evaluate phone phone"),
            Err(SessionError::Scenario(_))
        ));
    }

    #[test]
    fn removal_after_freeing_a_resource_updates_the_map() {
        let mut session = Session::default();
        run(&mut session, EXAMPLE);

        run(
            &mut session,
            &["remove route email-login", "remove resource phone"],
        );

        let listed = run(&mut session, &["list"]);
        assert!(!listed.iter().any(|line| line.contains("phone ")));
        assert!(listed
            .contains(&"route email-recovery email recovery-sheet Email recovery".to_string()));
    }
}
