// SPDX-License-Identifier: MIT

// Loading macros must be done at the crate root
#[macro_use]
extern crate log;

#[macro_use]
extern crate clap;

mod app;
mod config;

use app::run;
use config::Options;

fn main() {
    if let Some(app_options) = Options::parse() {
        run(app_options);
    }
}
