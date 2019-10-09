// SPDX-License-Identifier: MIT

// Loading macros must be at the crate root
#[macro_use]
extern crate log;

mod app;
mod config;

// use std::io;
use app::*;
use config::parse_flags;

fn main() {
    let matches = parse_flags().unwrap();

    if matches.value_of("list").is_some() {
        generate_gitignore();

        return;
    };

    if let Some(values) = matches.values_of("template") {
/*         for template in values {
 *             if template in global_list {
 *                 // Get template file path
 *             }
 *         }
 *
 *         // Trim then merge template files
 *         return; */
    };
}
