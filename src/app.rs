// SPDX-License-Identifier: MIT

extern crate git2;

use clap::ArgMatches;
use git2::Repository;

use std::collections::btree_map::BTreeMap;
use std::fs::File;
use std::io::{Error, Read};

const REPO_PARENT_DIR: &str = "~/.cache/ignore-ng/repos/";

// const DEFAULT_REPO_DIR: &str = format!("{}/{}", REPO_PARENT_DIR, DEFAULT_REPO_NAME);
const DEFAULT_GITIGNORE_REPO: &str = "https://github.com/github/gitignore";
const DEFAULT_REPO_NAME: &str = "github/gitignore";

// TODO: populate this
pub fn generate_gitignore(matches: &ArgMatches) -> Result<(), std::io::Error> {
    let available_templates =
        parse_templates(matches).expect("Failed to parse the template arguments");

    // Scan through gitignore dir
    // File::open();
    Ok(())
}

// TODO: populate this
pub fn list_templates() -> Result<(), std::io::Error> {
    // Read from global_list
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
fn update_gitignore_repo(repo_dir: &str, repo_url: &str) -> Result<(), git2::Error> {
    match Repository::open(repo_dir) {
        Ok(_repo) => {
            /* repo.stash_save(signature, flags)?;
             * repo.stash_drop(index)?; */

            Ok(())
        }
        Err(_) => {
            info!("Repository not cached locally, cloning");

            match Repository::clone_recurse(repo_url, repo_dir) {
                Ok(_) => {
                    debug!("Repository cloned from upstream");

                    Ok(())
                }
                Err(err) => {
                    warn!("Failed to clone: {} into: {}", repo_url, repo_dir);

                    Err(err)
                }
            }
        }
    }
}

// TODO: populate this
fn update_global_list(repo_dir: &str, global_list: &BTreeMap<&str, &str>) {
    // Store template name & path in hashmap
}
