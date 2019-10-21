// SPDX-License-Identifier: MIT

// Loading macros must be done at the crate root
#[macro_use]
extern crate log;

#[macro_use]
extern crate clap;

mod app;
mod config;

use app::{generate_gitignore, list_templates, update_gitignore_repo};
use config::Options;

fn main() {
    if let Some(mut app_options) = Options::parse() {
        if app_options.update_repo {
            update_gitignore_repo(&app_options).expect("Error updating gitignore repo")
        }

        if app_options.list_templates {
            list_templates(&mut app_options)
        }

        if app_options.generate_gitignore {
            generate_gitignore(&mut app_options).expect("Error generating .gitignore file");
        }

        app_options.save();
    }
}
