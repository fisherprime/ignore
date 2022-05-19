// SPDX-License-Identifier: MIT

#![forbid(unsafe_code)]
#![warn(missing_docs)]
// NOTE: unneeded, this is not a library.
// #![warn(missing_doc_code_examples)]

//! `ignore` is a collection of methods and items used to generate `gitignore` files.
//!
//! This crate consolidates locally cached `gitignore` templates into a `gitignore` file.

// Loading macros must be done at the crate root.
#[macro_use]
extern crate log;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate lazy_static;

mod app;
mod config;
mod errors;
mod utils;

use app::run;
use config::runtime::RuntimeConfig;

/// This is the entry point for `ignore`'s binary.
///
/// This function sets up the runtime environment [`RuntimeConfig`] then executes the specified operation.
fn main() {
    RuntimeConfig::default()
        .load()
        .map(|app_conf| {
            run(app_conf).unwrap_or_else(|err| error!("Application error: {}", err))
        })
        .unwrap_or_else(|err| error!("Application error: {}", err));
}
