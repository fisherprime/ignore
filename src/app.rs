// SPDX-License-Identifier: MIT

//! The app module defines elements that perform the user-availed tasks.

extern crate git2;

/* `self::`` doesn't work here.
 *
 * `super::` and `crate::` work.
 * Note: `super::` & `self::` are relative to the current module while `crate::` is relative to the
 * crate root.
 */
use crate::config::{Operation, Options, RepoDetails};

// use git2::{Object, Repository};
// use std::collections::hash_map::HashMap;
use git2::build::CheckoutBuilder;
use git2::Repository;
use std::collections::btree_map::BTreeMap;
use std::error::Error;
use std::fs::{self, DirBuilder, DirEntry, File, OpenOptions};
use std::io;
use std::io::prelude::*;
use std::path::Path;

// Macro used to reduce repetition when defining a cached repository's absolute path.
macro_rules! absolute_repo_path {
    ($parent:expr, $base:expr) => {
        format!(
            "{}/{}",
            $parent.config.repo.repo_parent_dir, $base.repo_path
        );
    };
}

/// Binary tree hash-map alias for simplicity.
type TemplatePaths = BTreeMap<String, Vec<String>>;

/// Handles the execution of ignore's functions.
///
/// Using the parsed runtime config [`Options`], runs a task specified by ignore's arguments then
/// overwrites the config file.
/// This function returns an error to the calling function on occurrence.
///
/// # Examples
///
/// ```
/// // mod app;
/// // mod config;
///
/// use crate::app::run;
/// use crate::config::Options;
///
/// Options::parse().map(|opts| {
///     run(opts)
///         .unwrap_or_else(|err| panic!("Application error: {}", err))
/// })
/// ```
pub fn run(mut app_options: Options) -> Result<(), Box<dyn Error>> {
    if app_options.needs_update {
        update_gitignore_repos(&app_options)?;

        if app_options.operation == Operation::UpdateRepo {
            app_options.save_config()?;
            return Ok(());
        }
    }

    match app_options.operation {
        Operation::GenerateGitignore => generate_gitignore(&mut app_options)?,
        Operation::ListTemplates => list_templates(&mut app_options)?,
        Operation::UpdateRepo => update_gitignore_repos(&app_options)?,
        Operation::Else => info!("No operation specified, this shouldn't have happened"),
    }

    app_options.save_config()?;

    Ok(())
}

/// Consolidates locally cached gitignore template files.
///
/// This function calls [`parse_templates`] (template argument parsing) then
/// [`concatenate_templates`] (template consolidation) for the user defined gitignore template
/// arguments, yielding a consolidated gitignore file.
///
/// # Examples
///
/// **Requires the user specify the `template` argument.**
///
/// ```
/// // mod app;
/// // mod config;
///
/// use app::generate_gitignore;
/// use config::Options;
///
/// Options::parse()
///     .map(|opts| generate_gitignore(&mut opts))
///     .unwrap_or_else(|err| panic!("Application error: {}", err))
///
/// ```
fn generate_gitignore(app_options: &mut Options) -> Result<(), Box<dyn Error>> {
    info!("Generating gitignore");

    let mut consolidation_file: File;

    let consolidation_string: String;

    let available_templates = parse_templates(app_options)?;
    debug!("Available templates: {:?}", available_templates);

    let result = {
        consolidation_string = concatenate_templates(available_templates)?;
        !consolidation_string.is_empty()
    };

    if result {
        consolidation_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&app_options.output_file)?;
        debug!("Opened and/or created gitignore consolidation file");

        consolidation_file.set_len(0)?;
        consolidation_file.write_all(consolidation_string.as_bytes())?;
        info!("Generated gitignore: {}", app_options.output_file);

        return Ok(());
    }

    warn!(
        "Specified template(s) could not be located (names are case sensitive): {:?}",
        app_options.templates
    );

    Ok(())
}

/// Concatenates gitignore template files specified by the user.
///
/// This function acts on [`TemplatePaths`] for the template arguments specified by a user.
/// The filespaths listed in the [`TemplatePaths`] are then consolidated into a single file.
fn concatenate_templates(available_templates: TemplatePaths) -> Result<String, Box<dyn Error>> {
    let delimiter = "# ----";

    let mut consolidation_string = String::new();

    if available_templates.is_empty() {
        return Ok(consolidation_string);
    }

    consolidation_string += "#\n# .gitignore\n#\n";

    // Iterate over template_paths, opening necessary file & concatenating them.
    for (template, file_paths) in available_templates {
        let file_paths = &file_paths;

        let mut template_string = format!("\n# {}\n{}\n", template, delimiter);

        let mut template_vec = Vec::<String>::new();

        for file_path in file_paths {
            debug!("Parsing: {}", file_path);

            match File::open(file_path) {
                Ok(mut template_file) => {
                    let mut temp_string = String::new();

                    template_file.read_to_string(&mut temp_string)?;
                    template_vec.push(temp_string.to_string());

                    debug!(
                        "Appended {} content to {} template vector",
                        file_path, template
                    );
                }
                Err(err) => {
                    error!("Error opening .gitignore template file: {}", err);
                    continue;
                }
            };
        }

        if template_vec.is_empty() {
            continue;
        }

        template_vec.sort();
        template_vec.dedup();

        if template_vec.len().gt(&1) {
            for temp_string in template_vec {
                // TODO: replace with per file deduplication_logic.
                template_string += &temp_string;
                // TODO: end replace with per file deduplication_logic.
            }
        } else {
            template_string += &template_vec.pop().unwrap();
        }

        template_string += format!("{}\n", delimiter).as_str();
        consolidation_string += template_string.as_str();
    }

    Ok(consolidation_string)
}

/// Lists the names of projects, tools, languages, ... with cached gitignore templates.
fn list_templates(app_options: &mut Options) -> Result<(), Box<dyn Error>> {
    info!("Listing available templates");

    let list_width = 6;

    let mut list_string = String::new();

    let mut key_vector: Vec<String>;

    let template_paths = generate_template_paths(app_options)?;

    key_vector = template_paths.keys().cloned().collect();
    key_vector.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

    for (index, key) in key_vector.iter().enumerate() {
        list_string += key;
        list_string += " ";

        if index % list_width == 0 {
            list_string += "\n";
        }
    }
    println!("{}", list_string);

    debug!("Done listing available templates");

    Ok(())
}

/// Generates [`TemplatePaths`] for the available gitignore template arguments supplied by a user.
///
/// This function generates a [`TemplatePaths`] item for the available gitignore template files
/// desired by a user.
/// Using the output of [`generate_template_paths`], the [`TemplatePaths`] is filtered to include
/// entries explicitly requested by the user.
fn parse_templates(app_options: &mut Options) -> Result<TemplatePaths, Box<dyn Error>> {
    debug!("Parsing template options");

    let mut available_templates = TemplatePaths::new();

    let template_list = app_options.templates.clone();

    let template_paths = generate_template_paths(app_options)?;

    /* template_paths = match sort_template_paths(&template_paths) {
     *     Some(sort) => sort,
     *     None => panic!("Template file paths hash map not sorted"),
     * };
     * debug!("Sorted template hash: {:?}", template_paths); */

    for template in template_list {
        // If template exists
        if let Some(t_paths) = template_paths.get(&template) {
            *available_templates.entry(template).or_default() = t_paths.to_vec();
        }
    }

    debug!("Selected available template options");

    Ok(available_templates)
}

/// Updates the cached gitignore template repositories (git only).
///
/// This function fetches and merges the latest HEAD for an existing git repository, cloning one if
/// not locally cached.
/// This operation will not update a repository if it hasn't reached staleness (as defined by the
/// const REPO_UPDATE_LIMIT) & the update operation isn't desired by the user.
///
/// REF: [github/nabijaczleweli](https://github.com/nabijaczleweli/cargo-update/blob/master/src/ops/mod.rs)
fn update_gitignore_repos(app_options: &Options) -> Result<(), Box<dyn Error>> {
    info!("Updating gitignore repo(s)");

    let mut checkout = CheckoutBuilder::new();

    for repo_det in app_options.config.repo.repo_dets.iter() {
        /* let repo: Repository;
         * let fetch_head: Object; */

        if !repo_det.auto_update && app_options.operation != Operation::UpdateRepo {
            continue;
        }

        let absolute_repo_path = absolute_repo_path!(app_options, repo_det);

        match Repository::discover(&absolute_repo_path) {
            Ok(repo) => {
                debug!("Repository is cached locally: {}", repo_det.repo_path);
                repo.find_remote("origin")?.fetch(&["master"], None, None)?;

                let fetch_head = repo
                    .find_reference("FETCH_HEAD")?
                    .peel(git2::ObjectType::Any)?;
                repo.reset(&fetch_head, git2::ResetType::Hard, Some(&mut checkout))?;
            }
            Err(_) => {
                info!(
                    "Repository not cached locally, cloning: {}",
                    repo_det.repo_path
                );

                clone_repository(app_options, &repo_det)?;
            }
        };

        info!("Updated gitignore repo: {}", repo_det.repo_path);
    }

    Ok(())
}

/// Clones a repository into a local cache directory.
fn clone_repository(
    app_options: &Options,
    repo_det: &RepoDetails,
) -> Result<Repository, Box<dyn Error>> {
    let absolute_repo_path = absolute_repo_path!(app_options, repo_det);

    DirBuilder::new()
        .recursive(true)
        .create(&app_options.config.repo.repo_parent_dir)?;

    Ok(Repository::clone_recurse(
        &repo_det.repo_url,
        &absolute_repo_path,
    )?)
}

// Using a BTreeMap is faster.
/* fn sort_template_paths(
 *     unsorted_map: &HashMap<String, Vec<String>>,
 * ) -> Option<HashMap<String, Vec<String>>> {
 *     debug!("Sorting template paths hash");
 *     // debug!("Unsorted map: {:?}", unsorted_map);
 *
 *     let mut key_vector: Vec<String>;
 *
 *     let mut sorted_map = HashMap::<String, Vec<String>>::new();
 *
 *     key_vector = unsorted_map.keys().cloned().collect();
 *     key_vector.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
 *     debug!("Sorted keys: {:?}", key_vector);
 *
 *     for key in key_vector {
 *         let path = unsorted_map
 *             .get(&key)
 *             .expect("Error sorting template path B-tree map");
 *         debug!("{:?}", path);
 *         *sorted_map.entry(key).or_default() = path.clone();
 *     }
 *     debug!("Done sorting template paths hash");
 *     // debug!("Sorted map: {:?}", sorted_map);
 *
 *     Some(sorted_map)
 * } */

/// Generates a [`TemplatePaths`] item (binary tree hash-map of gitignore template filepaths.
///
/// This function calls the update_template_paths function that updates the TemplatePaths hash-map.
fn generate_template_paths(app_options: &mut Options) -> Result<TemplatePaths, Box<dyn Error>> {
    let mut template_paths = TemplatePaths::new();

    for repo_det in app_options.config.repo.repo_dets.iter() {
        if repo_det.ignore {
            continue;
        }

        let absolute_repo_path = absolute_repo_path!(app_options, repo_det);

        // If the repository doesn't exist
        if !Path::new(&absolute_repo_path).is_dir() {
            clone_repository(&app_options, &repo_det)?;
        };

        update_template_paths(&Path::new(&absolute_repo_path), &mut template_paths)?;
        debug!("Template hash: {:?}", template_paths);
    }

    Ok(template_paths)
}

/// Populates a [`TemplatePaths`] item with filepath entries.
///
/// This function recurses on the contents of the cached gitignore template repositories, appending
/// filepath entries to the [`TemplatePaths`] hash-map for all available templates.
fn update_template_paths(dir: &Path, template_paths: &mut TemplatePaths) -> io::Result<()> {
    debug!(
        "Updating template file paths, dir: {}",
        dir.as_os_str().to_str().unwrap()
    );

    // Store template name & path in hashmap.
    for entry in fs::read_dir(dir)? {
        let entry = entry?;

        let entry_path_string: String;

        if ignore_file(&entry) {
            continue;
        }

        let entry_path = entry.path();
        entry_path_string = String::from(entry_path.clone().into_os_string().to_str().unwrap());

        if entry_path.is_dir() {
            update_template_paths(&entry_path, template_paths)?;
            debug!("Dir: {}", &entry_path_string);

            continue;
        }

        let template = template_paths
            .entry(remove_filetype(entry).unwrap())
            .or_default();

        template.push(entry_path_string);
    }

    debug!(
        "Done updating template file paths, dir: {}",
        dir.as_os_str().to_str().unwrap()
    );

    Ok(())
}

/// Removes the filetype from a pathname.
///
/// This function removes the filetype after the `.` from a file's basename.
fn remove_filetype(entry: DirEntry) -> Option<String> {
    // TODO: refine_filetype_removal, check for the existence of one, ...
    let t_filename = entry.file_name();

    #[allow(clippy::single_char_pattern)]
    let t_filename_split = t_filename
        .to_str()
        .unwrap()
        .split(".")
        .collect::<Vec<&str>>();

    Some(t_filename_split[0].to_string())

    // TODO: end refine_filetype_removal.
}

/// Checks whether a directory/file is hidden.
fn is_hidden(entry: &DirEntry) -> bool {
    #[allow(clippy::single_char_pattern)]
    entry
        .file_name()
        .to_str()
        .map(|f_name| f_name.starts_with("."))
        .unwrap_or(false)
}

/// Checks whether a file should be ignored during [`TemplatePaths`] population.
fn ignore_file(entry: &DirEntry) -> bool {
    // let ignores = Vec!["CHANGELOG", "LICENSE", "README", "CONTRIBUTING"];
    entry
        .file_name()
        .to_str()
        .map(|f_name| f_name.ends_with("md") || f_name.starts_with("LICENSE"))
        .unwrap_or(false)
        || is_hidden(&entry)
}
