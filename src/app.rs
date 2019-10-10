// SPDX-License-Identifier: MIT

extern crate git2;

// TODO: populate this
#[allow(dead_code)]
pub fn parse_languages_tools() {}

// TODO: populate this
#[allow(dead_code)]
pub fn generate_gitignore() {
// Scan through gitignore dir
}

// TODO: populate this
#[allow(dead_code)]
pub fn update_gitignore_repo() {}

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

