// SPDX-License-Identifier: MIT

//! The `errors` module defines `ignore`'s `Error` type, `ErrorKind` with their accompanying trait & method implementations.

use std::error::Error as StdErr;
use std::fmt::{Display, Formatter, Result};

#[allow(dead_code)]
#[derive(Debug)]
pub enum ErrorKind {
    /// User requested templates not found.
    MissingTemplates,

    /// No output generated for specified action.
    NoOutput,

    /// Error type for arbitrary (no fixed rule) errors.
    Other,
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    other_message: String,
    error: Option<Box<dyn StdErr>>,
}

/// Method implementations for [`errors::Error`].
impl Error {
    /// Creates a new [`errors::Error`] from a supplied [`errors::ErrorKind`] & [`Into<Box<dyn std::error::Error>>`] (type that can be converted into a boxable error struct).
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

    /// Returns the error's [`errors::ErrorKind`].
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

/// [`std::fmt::Display`] trait implementation for [`errors::Error`].
impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let message = match self.kind() {
            ErrorKind::MissingTemplates => {
                "None of the requested gitignore template(s) could be found"
            }
            ErrorKind::NoOutput => "No output was generated for the user specified operation",
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

/// [`std::error::Error`] trait implementation for [`errors::Error`].
impl StdErr for Error {
    fn source(&self) -> Option<&(dyn StdErr + 'static)> {
        match &self.error {
            Some(err) => Some(&**err),
            None => None,
        }
    }
}

/// [`std::convert::From<errors::ErrorKind>`] trait implementation for [`errors::Error].
impl From<ErrorKind> for Error {
    fn from(error_kind: ErrorKind) -> Self {
        Self {
            kind: error_kind,
            other_message: "".to_owned(),
            error: None,
        }
    }
}

/// [`std::convert::From<String>`] trait implementation for [`errors::Error].
impl From<String> for Error {
    fn from(message: String) -> Self {
        Self {
            kind: ErrorKind::Other,
            other_message: message,
            error: None,
        }
    }
}
