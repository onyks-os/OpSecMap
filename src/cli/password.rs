// Copyright (c) 2026 onyks-os
// SPDX-License-Identifier: AGPL-3.0-or-later
//!
//! Reading the password of an encrypted map file.
//!
//! The password is read from the terminal with echo turned off, not from
//! standard input, so it works when commands come from a file and never shows
//! on screen. It is held in [`Zeroizing`] and never appears in any message.

use std::fmt;

use zeroize::Zeroizing;

/// The environment variable that, when set, supplies the password instead of
/// the terminal.
///
/// It exists for tests and scripts with fictional maps: environment variables
/// can be read by other processes of the same user, so it is not meant for
/// real passwords.
pub(crate) const PASSWORD_ENV: &str = "OPSECMAP_PASSWORD";

/// An error while reading a password.
#[derive(Debug)]
pub(crate) enum PasswordError {
    /// The terminal could not be read, for example because there is none.
    Io(std::io::Error),
    /// The password is empty.
    Empty,
    /// The new password and its repetition differ.
    Mismatch,
}

/// Describes the error in one lowercase line, without the password.
impl fmt::Display for PasswordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "cannot read the password from the terminal: {error}"),
            Self::Empty => f.write_str("the password is empty"),
            Self::Mismatch => f.write_str("the two passwords do not match"),
        }
    }
}

/// Reads a password, from [`PASSWORD_ENV`] if it is set, otherwise from the
/// terminal after printing `prompt`.
///
/// The password is kept exactly as typed: spaces at either end are part of it.
///
/// # Errors
///
/// Returns [`PasswordError::Io`] if the terminal cannot be read, and
/// [`PasswordError::Empty`] if the password is empty.
pub(crate) fn read_password(prompt: &str) -> Result<Zeroizing<String>, PasswordError> {
    let password = Zeroizing::new(if let Ok(pwd) = std::env::var(PASSWORD_ENV) {
        pwd
    } else {
        rpassword::prompt_password(prompt).map_err(PasswordError::Io)?
    });
    if password.is_empty() {
        return Err(PasswordError::Empty);
    }
    Ok(password)
}

/// Reads a new password twice, for encrypting a map, and returns it if both
/// readings match.
///
/// With [`PASSWORD_ENV`] set, both readings take its value and always match.
///
/// # Errors
///
/// Returns the errors of [`read_password`], and [`PasswordError::Mismatch`] if
/// the two readings differ.
pub(crate) fn read_new_password() -> Result<Zeroizing<String>, PasswordError> {
    let new_password = read_password("New password: ")?;
    let repeat_password = read_password("Repeat password: ")?;
    if new_password != repeat_password {
        return Err(PasswordError::Mismatch);
    }
    Ok(new_password)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{read_new_password, read_password, PasswordError, PASSWORD_ENV};

    /// One test for every case that uses the environment variable: tests run in
    /// parallel, and the variable is shared by the whole process.
    #[test]
    fn the_environment_variable_supplies_the_password() {
        std::env::set_var(PASSWORD_ENV, " test password ");
        let password = read_password("unused prompt: ").expect("password must be read");
        assert_eq!(password.as_str(), " test password ");
        let new_password = read_new_password().expect("both readings must match");
        assert_eq!(new_password.as_str(), " test password ");

        std::env::set_var(PASSWORD_ENV, "");
        assert!(matches!(
            read_password("unused prompt: "),
            Err(PasswordError::Empty)
        ));
        assert!(matches!(read_new_password(), Err(PasswordError::Empty)));

        std::env::remove_var(PASSWORD_ENV);
    }

    #[test]
    fn messages_do_not_contain_a_password() {
        assert_eq!(PasswordError::Empty.to_string(), "the password is empty");
        assert_eq!(
            PasswordError::Mismatch.to_string(),
            "the two passwords do not match"
        );
    }
}
