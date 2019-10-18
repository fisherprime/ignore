// SPDX-License-Identifier: MIT

extern crate git2;

use crate::config::{Config, Options};
use git2::{Commit, MergeOptions, Repository, Signature, StashFlags, Time};

use chrono::Utc;
use std::collections::btree_map::BTreeMap;
use std::fs::{self, DirBuilder, DirEntry, File};
use std::io::{self, Read, Write};
use std::path::Path;

// TODO: validate this
pub fn generate_gitignore(app_config: &Config, app_options: &Options) -> Result<(), io::Error> {
    let available_templates: Vec<&str>;

    let mut consolidation_file = File::open(".gitignore-test")?;

    available_templates =
        parse_templates(&app_config, &app_options).expect("Failed to parse the template argument");

    // Iterate over template_paths, opening necessary file & concatenating them
    for file_path in available_templates {
        debug!("Parsing: {}", file_path);

        let mut template_string = String::new();

        let mut template_file = File::open(file_path).unwrap_or_else(|err| {
            // Prefer to break out of the loop
            panic!("Error opening gitignore template file: {:?}", err);
            // error!("Error openning gitignore template file: {:?}", err);
        });

        template_file.read_to_string(&mut template_string)?;
        consolidation_file.write_all(b"#----")?;
        consolidation_file.write_all(template_string.as_bytes())?;
        consolidation_file.write_all(b"#----")?;
    }

    Ok(())
}

// TODO: validate this
pub fn list_templates(app_config: &Config, app_options: &mut Options) {
    update_template_paths(
        &Path::new(&app_config.repo.repo_path),
        &mut app_options.template_paths,
    )
    .unwrap();

    for (key, value) in app_options.template_paths.iter() {
        println!("Template: {}, path: {}", key, value);
    }
}

// TODO: validate this
fn parse_templates(
    app_config: &Config,
    app_options: &Options,
) -> Result<Vec<&'static str>, io::Error> {
    let mut available_templates = Vec::<&str>::new();
    let mut template_paths = BTreeMap::<&str, &str>::new();

    let template_list = app_options.templates.clone();

    update_template_paths(&Path::new(&app_config.repo.repo_path), &mut template_paths).unwrap();
    debug!("Template path hash updated");

    for template in template_list {
        // If template exists
        if let Some(template_path) = template_paths.get(&template) {
            available_templates.push(template_path);
        }
    }

    Ok(available_templates)
}

// TODO: validate this
pub fn update_gitignore_repo(app_config: &Config) -> Result<(), git2::Error> {
    // Note: values in a scope are dropped in their order of creation
    let mut repo: Repository;
    let absolute_repo_path: String;

    let timezone_offset = 60 * 3;

    let fetch_head_commit: Commit;
    let current_head_commit: Commit;

    debug!("Updating gitignore repo");

    absolute_repo_path = format!(
        "{}/{}",
        app_config.repo.repo_parent_dir, app_config.repo.repo_path
    );

    repo = Repository::open(&absolute_repo_path).unwrap_or_else(|_| {
        info!("Repository not cached locally, cloning");

        let err_string = &format!(
            "Failed to clone: {} into: {:?}",
            app_config.repo.repo_url, app_config.repo.repo_path
        );

        DirBuilder::new()
            .recursive(true)
            .create(&app_config.repo.repo_parent_dir)
            .unwrap();

        Repository::clone_recurse(&app_config.repo.repo_url, &absolute_repo_path).expect(err_string)
        // info!("Repository cloned");
    });

    // Stash current state, then drop it
    info!("Stashing & clearing changes to repo");
    let result_oid = repo
        .stash_save(
            &Signature::new(
                "name",
                "e@mail.com",
                &Time::new(Utc::now().timestamp(), timezone_offset),
            )
            .unwrap(),
            "",
            Some(StashFlags::DEFAULT),
        )
        .unwrap_or_else(|_| {
            info!("Nothing to stash");
            git2::Oid::zero()
        });

    if result_oid == git2::Oid::zero() {
        return Ok(());
    }

    repo.stash_drop(0)?;

    // Pull changes from remote repository
    // REF: https://stackoverflow.com/questions/54100789/how-is-git-pull-done-with-the-git2-rs-rust-crate
    current_head_commit = repo.head()?.peel_to_commit()?;
    fetch_head_commit = repo
        .find_reference("FETCH_HEAD")
        .unwrap()
        .peel_to_commit()
        .unwrap();

    repo.merge_commits(
        &current_head_commit,
        &fetch_head_commit,
        Some(&MergeOptions::new()),
    )
    .unwrap();

    Ok(())
}

// TODO: populate this
fn update_template_paths(dir: &Path, template_paths: &mut BTreeMap<&str, &str>) -> io::Result<()> {
    // Store template name & path in hashmap
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.path().is_dir() {
            update_template_paths(&entry.path(), template_paths).unwrap();
        }
    }

    Ok(())
}

// TODO: validate this
#[allow(dead_code)]
fn is_hidden(entry: &DirEntry) -> bool {
    #[allow(clippy::single_char_pattern)]
    entry
        .file_name()
        .to_str()
        .map(|f_name| f_name.starts_with("."))
        .unwrap_or(false)
}
