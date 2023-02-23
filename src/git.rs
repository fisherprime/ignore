// SPDX-License-Identifier: MIT

//! The `git` module defines user-executable git tasks.

use crate::absolute_repo_path;
use crate::config::{configs::RepoConfig, runtime::Operation, runtime::RuntimeConfig};
use crate::errors::Error;

use git2::Repository;
use std::error::Error as StdErr;
use std::time::SystemTime;

use rayon::prelude::*;

/// Updates the cached gitignore template repositories (git only).
///
/// This function fetches and merges the latest `HEAD` for an existing git repository, cloning one if
/// not locally cached.
/// This operation will not update a repository if it hasn't reached staleness (as defined by
/// [`const REPO_UPDATE_LIMIT`]) & the update operation isn't desired by the user.
///
/// REF: [github/nabijaczleweli](https://github.com/nabijaczleweli/cargo-update/blob/master/src/ops/mod.rs)
#[allow(clippy::needless_late_init)]
pub fn update_gitignore_repos(app_conf: &mut RuntimeConfig) {
    info!("git: updating gitignore repo(s)");

    app_conf
        .config
        .repository
        .config
        .par_iter()
        .for_each(|conf| {
            let update_cond = !conf.url.is_empty()
                && (conf.auto_update || app_conf.operation == Operation::UpdateRepositories);
            if update_cond {
                if let Err(err) = update_repo(app_conf, conf) {
                    error!("{}", err);
                }
            }
        });

    app_conf.state.last_update = SystemTime::now()
}

fn update_repo(app_conf: &RuntimeConfig, conf: &RepoConfig) -> Result<(), Box<dyn StdErr>> {
    // fn update_repo(app_conf: &RuntimeConfig, conf: &RepoConfig) -> Box <dyn Future<Output =Result<(), Box<dyn StdErr>>> >{

    match Repository::discover(absolute_repo_path!(app_conf, conf)) {
        Ok(repo) => {
            use git2::build::CheckoutBuilder;
            debug!("git: updating cached repository {}", conf.path);

            // Work on repo's with the HEAD set to a branch.
            let head = repo.head()?;
            if !head.is_branch() {
                info!(
                    "git: gitignore repo's HEAD is not a branch, skipping {}",
                    conf.path
                )
            }

            // Get branch name from HEAD reference.
            match head.name() {
                Some(branch) => {
                    let mut remote = repo.find_remote("origin")?;
                    remote.fetch(&[branch], None, None)?;
                }
                None => return Err(Box::new(Error::from("invalid branch name"))),
            }

            let fetch_head = repo
                .find_reference("FETCH_HEAD")?
                .peel(git2::ObjectType::Any)?;

            let mut checkout = CheckoutBuilder::new();
            repo.reset(&fetch_head, git2::ResetType::Hard, Some(&mut checkout))?;
        }
        Err(_) => {
            info!("git: caching new repository {}", conf.path);
            fetch_repository(app_conf, conf)?;
        }
    };

    info!("git: updated gitignore repo {}", conf.path);

    Ok(())
}

/// Fetches a git repository for local caching.
pub fn fetch_repository(
    app_conf: &RuntimeConfig,
    conf: &RepoConfig,
) -> Result<Repository, Box<dyn StdErr>> {
    use std::fs::DirBuilder;

    info!("git: cloning gitignore repo {}", conf.path);

    DirBuilder::new()
        .recursive(true)
        .create(&app_conf.config.repository.cache_dir)?;

    // NOTE: Wrapped in `Ok` to allow for the conversion of `git::error::Error` to `Box<dyn std::error::Error>`.
    Ok(Repository::clone_recurse(
        &conf.url,
        absolute_repo_path!(app_conf, conf),
    )?)
}
