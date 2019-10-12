// SPDX-License-Identifier: MIT

extern crate git2;

use crate::config::Config;
use clap::ArgMatches;
use git2::Repository;

use std::collections::btree_map::BTreeMap;
use std::fs;
use std::fs::DirEntry;
use std::io::{Error, Read};

// TODO: populate this
pub fn generate_gitignore(matches: &ArgMatches, app_config: &Config) -> Result<(), std::io::Error> {
    let available_templates =
        parse_templates(matches).expect("Failed to parse the template arguments");

    // Iterate over global_list, opening necessary file & concatenating them
    // File::open();
    Ok(())
}

// TODO: populate this
pub fn list_templates(app_config: &Config) -> Result<(), std::io::Error> {
    // Iterate over global_list printing keys
    Ok(())
}

// TODO: populate this
fn parse_templates(matches: &ArgMatches) -> Result<Vec<&'static str>, Error> {
    let mut available_templates: Vec<&str>;

    if let Some(values) = matches.values_of("template") {
        /*                 for template in values {
         *                     // If template exists
         *                     if template in global_list {
         *                         // Get template file path
         *                     }
         *                 }
         *
         *                 // Trim then merge template files
         *                 return; */
    };

    Ok(available_templates)
}

// TODO: populate this
fn update_gitignore_repo(config: &Config) -> Result<(), git2::Error> {
    match Repository::open(config.repo_path) {
        Ok(repo) => {
            /* repo.stash_save(signature, flags)?;
             * repo.stash_drop(index)?; */

            Ok(())
        }
        Err(_) => {
            info!("Repository not cached locally, cloning");

            match Repository::clone_recurse(&config.repo_url, &config.repo_path) {
                Ok(_) => {
                    info!("Repository cloned from upstream");

                    Ok(())
                }
                Err(err) => {
                    error!(
                        "Failed to clone: {} into: {:?}",
                        config.repo_url, config.repo_path
                    );

                    Err(err)
                }
            }
        }
    }
}

// TODO: populate this
fn update_global_list(dir: &str, global_list: &BTreeMap<&str, &str>) -> Result<(), std::io::Error> {
    // Store template name & path in hashmap

    for entry in fs::read_dir(dir) {
        let file = entry?;

        if fs::metadata(file.path).unwrap().is_dir() {
            update_global_list(file, global_list).unwrap();
        }
    }

    Ok(())
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|f_name| f_name.starts_with("."))
        .unwrap_or(false)
}
