// Copyright (c) 2026 onyks-os
// SPDX-License-Identifier: AGPL-3.0-or-later
//!
//! Parsing of one input line into a validated [`Command`].
//!
//! Words are separated by whitespace; ID lists are separated by commas, and `-`
//! stands for an empty list. A label is the rest of the line, with runs of
//! whitespace collapsed to single spaces.

use std::fmt;
use std::str::SplitWhitespace;

use opsecmap::{DocumentationCoverage, ResourceError, ResourceId, RouteError, RouteId};

/// The command reference printed by `help`.
pub(crate) const HELP: &[&str] = &[
    "resource <id> <label...>                        add a resource",
    "route <id> <target> <prereq,...> <label...>     add a route; all prerequisites are required",
    "coverage <id> <unknown|incomplete|complete>     set how completely the routes to <id> are documented",
    "evaluate <initial,...|-> <excluded,...|->       evaluate a scenario; - means none",
    "remove resource <id>                            remove a resource no route refers to",
    "remove route <id>                               remove a route",
    "list                                            print the map as commands",
    "help                                            print this reference",
    "quit                                            stop reading commands",
    "Blank lines and lines starting with # are ignored.",
];

/// One validated instruction read from the command line.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Command {
    /// `resource <id> <label...>`
    AddResource {
        /// Identifier of the new resource.
        id: ResourceId,
        /// Human-readable label.
        label: String,
    },
    /// `route <id> <target> <prereq,...> <label...>`
    AddRoute {
        /// Identifier of the new route.
        id: RouteId,
        /// Resource the route makes accessible.
        target: ResourceId,
        /// Resources required together.
        prerequisites: Vec<ResourceId>,
        /// Human-readable label.
        label: String,
    },
    /// `coverage <id> <unknown|incomplete|complete>`
    SetCoverage {
        /// Resource whose coverage changes.
        id: ResourceId,
        /// The new coverage.
        coverage: DocumentationCoverage,
    },
    /// `evaluate <initial,...|-> <excluded,...|->`
    Evaluate {
        /// Resources available at the start.
        initial: Vec<ResourceId>,
        /// Resources excluded throughout the evaluation.
        excluded: Vec<ResourceId>,
    },
    /// `remove resource <id>`
    RemoveResource {
        /// Resource to remove.
        id: ResourceId,
    },
    /// `remove route <id>`
    RemoveRoute {
        /// Route to remove.
        id: RouteId,
    },
    /// `list`
    List,
    /// `help`
    Help,
    /// `quit`
    Quit,
}

/// Why a line could not be turned into a command.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ParseError {
    /// The first word is not a known command.
    UnknownCommand(String),
    /// A required argument is missing; holds its name.
    MissingArgument(&'static str),
    /// The command received more words than it accepts.
    TooManyArguments,
    /// The coverage word is not `unknown`, `incomplete` or `complete`.
    InvalidCoverage(String),
    /// `remove` was followed by something other than `resource` or `route`.
    InvalidRemoveKind(String),
    /// An ID contains a comma or is `-`, which the list syntax reserves.
    ReservedId(String),
    /// A resource ID failed validation.
    InvalidResourceId(ResourceError),
    /// A route ID failed validation.
    InvalidRouteId(RouteError),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownCommand(name) => {
                write!(f, "unknown command \"{name}\"; type \"help\" for the list")
            }
            Self::MissingArgument(argument) => write!(f, "missing argument: {argument}"),
            Self::TooManyArguments => f.write_str("too many arguments"),
            Self::InvalidCoverage(value) => write!(
                f,
                "invalid coverage \"{value}\"; expected unknown, incomplete or complete"
            ),
            Self::InvalidRemoveKind(kind) => {
                write!(f, "cannot remove \"{kind}\"; expected resource or route")
            }
            Self::ReservedId(value) => write!(
                f,
                "invalid ID \"{value}\": IDs cannot contain ',' or be '-'"
            ),
            Self::InvalidResourceId(e) => write!(f, "{e}"),
            Self::InvalidRouteId(e) => write!(f, "{e}"),
        }
    }
}

/// Parses one input line.
///
/// Returns `Ok(None)` for a blank line or a `#` comment: there is nothing to run,
/// but nothing is wrong either.
pub(crate) fn parse(line: &str) -> Result<Option<Command>, ParseError> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Ok(None);
    }
    let mut words = line.split_whitespace();
    let Some(name) = words.next() else {
        return Ok(None);
    };

    let command = match name {
        "resource" => {
            let id = resource_id(next(&mut words, "id")?)?;
            let label = label(words)?;
            Command::AddResource { id, label }
        }
        "route" => {
            let id = route_id(next(&mut words, "id")?)?;
            let target = resource_id(next(&mut words, "target")?)?;
            let prerequisites = id_list(next(&mut words, "prerequisites")?)?;
            let label = label(words)?;
            Command::AddRoute {
                id,
                target,
                prerequisites,
                label,
            }
        }
        "coverage" => {
            let id = resource_id(next(&mut words, "id")?)?;
            let coverage = coverage(next(&mut words, "coverage")?)?;
            end(words)?;
            Command::SetCoverage { id, coverage }
        }
        "evaluate" => {
            let initial = id_list(next(&mut words, "initial")?)?;
            let excluded = id_list(next(&mut words, "excluded")?)?;
            end(words)?;
            Command::Evaluate { initial, excluded }
        }
        "remove" => {
            let kind = next(&mut words, "resource or route")?;
            let id = next(&mut words, "id")?;
            let command = match kind {
                "resource" => Command::RemoveResource {
                    id: resource_id(id)?,
                },
                "route" => Command::RemoveRoute { id: route_id(id)? },
                other => return Err(ParseError::InvalidRemoveKind(other.to_string())),
            };
            end(words)?;
            command
        }
        "list" => {
            end(words)?;
            Command::List
        }
        "help" => {
            end(words)?;
            Command::Help
        }
        "quit" => {
            end(words)?;
            Command::Quit
        }
        other => return Err(ParseError::UnknownCommand(other.to_string())),
    };
    Ok(Some(command))
}

/// Returns the keyword `coverage` accepts for `value`; the inverse of parsing.
pub(crate) fn coverage_keyword(value: DocumentationCoverage) -> &'static str {
    match value {
        DocumentationCoverage::Unknown => "unknown",
        DocumentationCoverage::Incomplete => "incomplete",
        DocumentationCoverage::Complete => "complete",
    }
}

/// Takes the next word, or reports the missing argument by name.
fn next<'a>(
    words: &mut SplitWhitespace<'a>,
    argument: &'static str,
) -> Result<&'a str, ParseError> {
    words.next().ok_or(ParseError::MissingArgument(argument))
}

/// Joins the remaining words into a label; at least one word is required.
fn label(words: SplitWhitespace<'_>) -> Result<String, ParseError> {
    let label = words.collect::<Vec<&str>>().join(" ");
    if label.is_empty() {
        return Err(ParseError::MissingArgument("label"));
    }
    Ok(label)
}

/// Fails when any word remains.
fn end(mut words: SplitWhitespace<'_>) -> Result<(), ParseError> {
    match words.next() {
        Some(_) => Err(ParseError::TooManyArguments),
        None => Ok(()),
    }
}

/// Rejects IDs that would be ambiguous in the list syntax.
fn check_not_reserved(text: &str) -> Result<(), ParseError> {
    if text == "-" || text.contains(',') {
        return Err(ParseError::ReservedId(text.to_string()));
    }
    Ok(())
}

fn resource_id(text: &str) -> Result<ResourceId, ParseError> {
    check_not_reserved(text)?;
    ResourceId::new(text.to_string()).map_err(ParseError::InvalidResourceId)
}

fn route_id(text: &str) -> Result<RouteId, ParseError> {
    check_not_reserved(text)?;
    RouteId::new(text.to_string()).map_err(ParseError::InvalidRouteId)
}

/// Parses `a,b,c`, or `-` for an empty list. An empty item such as in `a,,b`
/// fails validation.
fn id_list(text: &str) -> Result<Vec<ResourceId>, ParseError> {
    if text == "-" {
        return Ok(Vec::new());
    }
    text.split(',')
        .map(|item| ResourceId::new(item.to_string()).map_err(ParseError::InvalidResourceId))
        .collect()
}

fn coverage(text: &str) -> Result<DocumentationCoverage, ParseError> {
    match text {
        "unknown" => Ok(DocumentationCoverage::Unknown),
        "incomplete" => Ok(DocumentationCoverage::Incomplete),
        "complete" => Ok(DocumentationCoverage::Complete),
        other => Err(ParseError::InvalidCoverage(other.to_string())),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{coverage_keyword, parse, Command, ParseError};
    use opsecmap::{DocumentationCoverage, ResourceError, ResourceId, RouteId};

    fn id(value: &str) -> ResourceId {
        ResourceId::new(value.to_string()).expect("test ID must be valid")
    }

    fn route_id(value: &str) -> RouteId {
        RouteId::new(value.to_string()).expect("test route ID must be valid")
    }

    fn parsed(line: &str) -> Command {
        parse(line)
            .expect("line must parse")
            .expect("line must contain a command")
    }

    #[test]
    fn blank_lines_and_comments_are_skipped() {
        for line in ["", "   ", "\t", "# a comment", "   # indented comment"] {
            assert_eq!(parse(line), Ok(None), "{line:?}");
        }
    }

    #[test]
    fn resource_label_is_the_rest_of_the_line() {
        assert_eq!(
            parsed("resource phone   Test   phone "),
            Command::AddResource {
                id: id("phone"),
                label: "Test phone".to_string(),
            }
        );
    }

    #[test]
    fn route_reads_target_prerequisites_and_label() {
        assert_eq!(
            parsed("route email-login email password-store,phone Email login"),
            Command::AddRoute {
                id: route_id("email-login"),
                target: id("email"),
                prerequisites: vec![id("password-store"), id("phone")],
                label: "Email login".to_string(),
            }
        );
    }

    #[test]
    fn coverage_accepts_each_keyword() {
        for value in [
            DocumentationCoverage::Unknown,
            DocumentationCoverage::Incomplete,
            DocumentationCoverage::Complete,
        ] {
            let line = format!("coverage email {}", coverage_keyword(value));
            assert_eq!(
                parsed(&line),
                Command::SetCoverage {
                    id: id("email"),
                    coverage: value,
                }
            );
        }
    }

    #[test]
    fn evaluate_reads_two_lists_and_dash_means_empty() {
        assert_eq!(
            parsed("evaluate recovery-sheet,seed phone"),
            Command::Evaluate {
                initial: vec![id("recovery-sheet"), id("seed")],
                excluded: vec![id("phone")],
            }
        );
        assert_eq!(
            parsed("evaluate - -"),
            Command::Evaluate {
                initial: Vec::new(),
                excluded: Vec::new(),
            }
        );
    }

    #[test]
    fn remove_reads_kind_and_id() {
        assert_eq!(
            parsed("remove resource phone"),
            Command::RemoveResource { id: id("phone") }
        );
        assert_eq!(
            parsed("remove route email-login"),
            Command::RemoveRoute {
                id: route_id("email-login")
            }
        );
    }

    #[test]
    fn commands_without_arguments() {
        assert_eq!(parsed("list"), Command::List);
        assert_eq!(parsed("help"), Command::Help);
        assert_eq!(parsed("quit"), Command::Quit);
    }

    #[test]
    fn unknown_command_reports_its_name() {
        assert_eq!(
            parse("delete phone"),
            Err(ParseError::UnknownCommand("delete".to_string()))
        );
    }

    #[test]
    fn missing_arguments_are_named() {
        for (line, argument) in [
            ("resource", "id"),
            ("resource phone", "label"),
            ("route email-login", "target"),
            ("route email-login email", "prerequisites"),
            ("route email-login email phone", "label"),
            ("coverage email", "coverage"),
            ("evaluate", "initial"),
            ("evaluate -", "excluded"),
            ("remove", "resource or route"),
            ("remove route", "id"),
        ] {
            assert_eq!(
                parse(line),
                Err(ParseError::MissingArgument(argument)),
                "{line:?}"
            );
        }
    }

    #[test]
    fn extra_words_are_rejected_where_no_label_is_expected() {
        for line in [
            "coverage email complete now",
            "evaluate - - -",
            "remove route email-login now",
            "list all",
            "help me",
            "quit now",
        ] {
            assert_eq!(parse(line), Err(ParseError::TooManyArguments), "{line:?}");
        }
    }

    #[test]
    fn invalid_coverage_and_remove_kind_report_the_word() {
        assert_eq!(
            parse("coverage email full"),
            Err(ParseError::InvalidCoverage("full".to_string()))
        );
        assert_eq!(
            parse("remove map phone"),
            Err(ParseError::InvalidRemoveKind("map".to_string()))
        );
    }

    #[test]
    fn reserved_ids_are_rejected() {
        assert_eq!(
            parse("resource a,b Label"),
            Err(ParseError::ReservedId("a,b".to_string()))
        );
        assert_eq!(
            parse("route - email phone Label"),
            Err(ParseError::ReservedId("-".to_string()))
        );
    }

    #[test]
    fn empty_item_in_id_list_is_invalid() {
        assert_eq!(
            parse("evaluate a,,b -"),
            Err(ParseError::InvalidResourceId(ResourceError::InvalidId))
        );
    }
}

/// Properties of the parser over arbitrary input lines. `make fuzz` runs them, with
/// many more cases on the weekly schedule.
#[cfg(test)]
mod properties {
    use super::{parse, Command};
    use opsecmap::ResourceId;
    use proptest::prelude::*;

    proptest! {
        /// Any line, including control characters and broken structure, yields a
        /// command or an error: never a panic.
        #[test]
        fn parse_never_panics(line in "\\PC{0,200}") {
            let _ = parse(&line);
        }

        /// Lines assembled from command words reach the argument branches far more
        /// often than random text does; they must not panic either.
        #[test]
        fn parse_never_panics_on_command_like_lines(
            words in prop::collection::vec(
                prop_oneof![
                    Just("resource"), Just("route"), Just("coverage"), Just("evaluate"),
                    Just("remove"), Just("list"), Just("help"), Just("quit"),
                    Just("-"), Just(","), Just("a,b"), Just("a,,b"), Just("complete"),
                    Just("#"), Just("x"),
                ],
                0..8,
            )
        ) {
            let _ = parse(&words.join(" "));
        }

        /// A resource line with a valid ID and label parses to exactly that ID and
        /// label, with runs of spaces in the label collapsed to one.
        #[test]
        fn resource_line_keeps_id_and_label(
            id in "[a-z][a-z0-9-]{0,15}",
            words in prop::collection::vec("[A-Za-z0-9]{1,8}", 1..5),
            gap in " {1,3}",
        ) {
            let line = format!("resource {id}{gap}{}", words.join(&gap));
            let id = ResourceId::new(id).map_err(|e| TestCaseError::fail(e.to_string()))?;
            let expected = Command::AddResource { id, label: words.join(" ") };
            prop_assert_eq!(parse(&line), Ok(Some(expected)));
        }
    }
}
