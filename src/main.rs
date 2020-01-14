// SPDX-License-Identifier: MIT

#![warn(missing_docs)]

// NOTE: unneeded, this is not a library.
// #![warn(missing_doc_code_examples)]

//! The ignore-ng crate generates gitignore files.
//!
//! This crate consolidates locally cached gitignore templates into a gitignore file.

// Loading macros must be done at the crate root.
#[macro_use]
extern crate log;

#[macro_use]
extern crate clap;

mod app;
mod config;

use app::run;
use config::Options;

/// This is the entry point for the crate's binary.
///
/// This function initiates the setting up of the running environment then calls the function to
/// run the underlying logic.
fn main() {
    Options::parse()
        .map(|app_options| {
            run(app_options).unwrap_or_else(|err| panic!("Application error: {}", err))
        })
        .unwrap_or_else(|err| panic!("Application error: {}", err));
}
