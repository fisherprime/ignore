// SPDX-License-Identifier: MIT

// Loading macros must be done at the crate root
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
