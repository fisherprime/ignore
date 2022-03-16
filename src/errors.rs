// SPDX-License-Identifier: MIT

//! The `errors` module defines `ignore`'s [`Error`] type, [`ErrorKind`] with their accompanying trait & method implementations.

use std::error::Error as StdErr;
use std::fmt::{Display, Formatter, Result};

/// `enum` containing the possible kinds of errors for `ignore`.
#[allow(dead_code)]
#[derive(Debug)]
pub enum ErrorKind {
    /// `dirs-next` failed to return the user's config directory.
    LocateConfigDir,

    /// User requested templates not found.
    MissingTemplates,

    /// No output generated for specified action.
    NoOutput,

    /// Unknown completion shell.
    UnknownCompletionShell,

    /// Error type for arbitrary (no fixed rule) errors.
    Other,
}

/// `struct` containing `ignore`'s error content.
#[derive(Debug)]
pub struct Error {
    /// The kind of error as enumerated in [`ErrorKind`].
    kind: ErrorKind,

    /// The message for an [`ErrorKind::Other`] error.
    other_message: String,

    // FIXME: Look into moving other_message into error; the `Option` will have to go.
    /// Optional field containing error resulting in this error.
    error: Option<Box<dyn StdErr + Send + Sync>>,
}

/// Method implementations for [`Error`].
impl Error {
    /// Creates a new [`Error`] from a supplied [`ErrorKind`] & `Into<Box<dyn std::error::Error>>`
    /// (type that can be converted into a boxable error struct).
    #[allow(dead_code)]
    pub fn new<T>(error_kind: ErrorKind, error_source: T) -> Self
    where
        T: Into<Box<dyn StdErr + Send + Sync>>,
    {
        Self {
            kind: error_kind,
            other_message: "".to_owned(),
            error: Some(error_source.into()),
        }
    }

    /// Returns the error's [`ErrorKind`].
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let message = match self.kind() {
            ErrorKind::MissingTemplates => {
                "None of the requested gitignore template(s) could be found"
            }
            ErrorKind::NoOutput => "No output was generated for the user specified operation",
            ErrorKind::LocateConfigDir => "Failed to locate config directory",
            ErrorKind::UnknownCompletionShell => "Unknown completion shell",
            ErrorKind::Other => {
                if self.other_message.is_empty() {
                    "User defined error with no payload encountered"
                } else {
                    &self.other_message
                }
            }
        };
        write!(f, "{}", &message)
    }
}

impl StdErr for Error {
    fn source(&self) -> Option<&(dyn StdErr + 'static)> {
        match &self.error {
            Some(err) => Some(&**err),
            None => None,
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(error_kind: ErrorKind) -> Self {
        Self {
            kind: error_kind,
            other_message: "".to_owned(),
            error: None,
        }
    }
}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Self {
            kind: ErrorKind::Other,
            other_message: message,
            error: None,
        }
    }
}
