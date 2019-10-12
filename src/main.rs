// SPDX-License-Identifier: MIT

// Loading macros must be done at the crate root
#[macro_use]
extern crate log;

#[macro_use]
extern crate clap;

mod app;
mod config;

use app::{list_templates, generate_gitignore};
use config::parse_flags;

fn main() {
    let matches = parse_flags().unwrap();

    if matches.value_of("list").is_some() {
        list_templates();

        return;
    };

    if matches.is_present("template") {
        generate_gitignore(&matches);
    }
}
