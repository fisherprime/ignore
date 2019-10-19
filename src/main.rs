// SPDX-License-Identifier: MIT

// Loading macros must be done at the crate root
#[macro_use]
extern crate log;

#[macro_use]
extern crate clap;

mod app;
mod config;

use app::{generate_gitignore, list_templates, update_gitignore_repo};
use config::Config;

fn main() {
    if let Some((app_config, mut app_options)) = Config::parse() {
        if app_options.update_repo {
            update_gitignore_repo(&app_config).expect("Error updating gitignore repo");
        }

        if app_options.list_templates {
            list_templates(&app_config, &mut app_options)
        }

        if app_options.generate_gitignore {
            generate_gitignore(&app_config, &mut app_options).expect("Error generating .gitignore file");
        }

        // app_config.update_config_file(&app_options.config_path);
    }
}
