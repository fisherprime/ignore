// SPDX-License-Identifier: MIT

#![warn(missing_docs)]

//! The ignore-ng crate generates gitignore files.
//!
//! This crate uses locally cached gitignore template definitions that are consolidated into a
//! gitignore file.

// Loading macros must be done at the crate root.
#[macro_use]
extern crate log;

#[macro_use]
extern crate clap;

mod app;
mod config;

use app::run;

fn main() {
    if let Err(err) = run() {
        panic!("Application error: {}", err)
    }
}
