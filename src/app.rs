// SPDX-License-Identifier: MIT

//! The `app` module defines elements that perform the user-availed tasks.

/* `self::`` doesn't work here.
 *
 * `super::` and `crate::` work.
 * Note: `super::` & `self::` are relative to the current module while `crate::` is relative to the
 * crate root.
 */
use crate::config::{config_file::RepoDetails, options::Operation, options::Options};
use crate::errors::{Error, ErrorKind};

use std::collections::btree_map::BTreeMap;
use std::error::Error as StdErr;
use std::fs::{self, DirEntry, File};
use std::io::{self, prelude::*};
use std::path::Path;

use git2::Repository;

// Macro used to reduce repetition when defining a cached repository's absolute path.
macro_rules! absolute_repo_path {
    ($parent:expr, $base:expr) => {
        format!(
            "{}/{}",
            $parent.config.repo.repo_parent_dir, $base.repo_path
        );
    };
}

/// `Binary tree hash-map` alias for simplicity.
type TemplatePaths = BTreeMap<String, Vec<String>>;

/// Const specifying the column limit to wrap an [`Operation::ListTemplates`] list line.
const TEMPLATE_LIST_OUTPUT_LIMIT: usize = 78;

/// Const specifying the file content delimiter used.
const FILE_CONTENT_DELIMITER: &str = "# ----";

/// Handles the execution of `ignore`'s functions.
///
/// Using the parsed [`Options`], this function runs a task specified by the user in `ignore`'s
/// arguments then updates the config file.
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
pub fn run(mut app_options: Options) -> Result<(), Box<dyn StdErr>> {
    if app_options.needs_update {
        update_gitignore_repos(&mut app_options)?;
        if app_options.operation == Operation::UpdateRepo {
            app_options.config.save_file()?;
            return app_options.state.save_file();
        }
    }

    match app_options.operation {
        Operation::GenerateGitignore => generate_gitignore(&mut app_options)?,
        Operation::ListTemplates => list_templates(&mut app_options)?,
        Operation::UpdateRepo => update_gitignore_repos(&mut app_options)?,
        Operation::Else => info!("No operation specified, this shouldn't have happened"),
    }

    app_options.config.save_file()?;
    app_options.state.save_file()
}

/// Consolidates locally cached gitignore template files.
///
/// This function calls [`parse_templates`] then [`concatenate_templates`]  for the user defined
/// gitignore template arguments, yielding a consolidated gitignore file.
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
fn generate_gitignore(app_options: &mut Options) -> Result<(), Box<dyn StdErr>> {
    use std::fs::OpenOptions;

    info!("Generating gitignore");

    let consolidation_string: String;

    let available_templates = parse_templates(app_options)?;
    debug!("Available templates: {:#?}", available_templates);

    consolidation_string = concatenate_templates(&app_options.templates, available_templates)?;

    let mut consolidation_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&app_options.output_file)?;
    debug!("Opened gitignore template consolidation file");

    consolidation_file.set_len(0)?;
    consolidation_file.write_all(consolidation_string.as_bytes())?;
    info!("Generated gitignore: {}", app_options.output_file);

    Ok(())
}

/// Concatenates gitignore template files specified by the user.
///
/// This function acts on a [`TemplatePaths`] item for the template arguments specified by a user,
/// consolidating the filespaths listed within the item.
fn concatenate_templates(
    requested_templates: &[String],
    available_templates: TemplatePaths,
) -> Result<String, Box<dyn StdErr>> {
    let mut consolidation_string = String::new();
    let mut return_string = String::new();
    let mut templates_used = String::new();

    if available_templates.is_empty() {
        warn!(
        "Neither of the specified template(s) could not be located (names are case sensitive): {:?}",
        requested_templates);
        return Err(Box::new(Error::from(ErrorKind::MissingTemplates)));
    }

    // Iterate over template_paths, opening necessary file & concatenating them.
    for (template, file_paths) in available_templates {
        let file_paths = &file_paths;

        let mut template_string = format!("\n# {}\n{}\n", template, FILE_CONTENT_DELIMITER);

        let mut template_vec = Vec::<String>::new();

        for file_path in file_paths {
            debug!("Parsing: {}", file_path);
            match File::open(file_path) {
                Ok(mut template_file) => {
                    let mut buffer = String::new();

                    template_file.read_to_string(&mut buffer)?;
                    template_vec.push(buffer.to_owned());

                    debug!(
                        "Appended {} content to {} template vector",
                        file_path, template
                    );
                }
                Err(err) => {
                    error!("Error opening gitignore template file: {}", err);
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
            let deduped_string = dedup_templates(&template, template_vec.as_mut())?;

            templates_used.push_str(&format!(" {}", template));
            template_string.push_str(&deduped_string);
        } else {
            templates_used.push_str(&format!(" {}", template));
            template_string.push_str(&template_vec[0]);
        }
        template_string.push_str(&format!("{}\n", FILE_CONTENT_DELIMITER));

        consolidation_string.push_str(&template_string);
    }

    if templates_used.is_empty() {
        warn!(
        "Neither of the specified template(s) could not be located (names are case sensitive): {:?}",
        requested_templates);
        return Err(Box::new(Error::from(ErrorKind::MissingTemplates)));
    }

    return_string.push_str("#\n# .gitignore\n#\n\n");
    return_string.push_str(&format!(
        "# Templates used:{}\n{}",
        templates_used, consolidation_string
    ));

    Ok(return_string)
}

/// Deduplicates gitignore template file content.
fn dedup_templates(
    template: &str,
    template_vec: &mut Vec<String>,
) -> Result<String, Box<dyn StdErr>> {
    // FIXME: Review this function for a better approach if any.
    // Iterating over all the lines for subsequent template files of a given technology seems
    // wasteful, they shouldn't be more than one so...

    info!("Deduplicating gitignore template entries for: {}", template);

    use lazy_static::lazy_static;
    use regex::Regex;

    // NOTE: recommended by the `regex` crate's developers to avoid recompilation of the regex rule
    // on subsequent runs.
    lazy_static! {
        static ref GITIGNORE_ENTRY_REGEX: Regex =
            Regex::new(r"[\*/!]").expect("Failed to compile gitignore entry regex");
    }

    let primary_content = template_vec[0].clone();
    let mut insert_string = String::new();

    for template_file in template_vec.iter().skip(1) {
        for line in template_file.lines() {
            let trimmed_line = line.trim();

            let invalid_line = {
                !GITIGNORE_ENTRY_REGEX.is_match(trimmed_line)
                    || primary_content.contains(trimmed_line)
                    || insert_string.contains(trimmed_line)
            };
            if invalid_line {
                continue;
            } else {
                if insert_string.is_empty() {
                    insert_string.push_str(&format!("{}\n", primary_content));
                    insert_string.push_str(&format!(
                        "# {}, supplementary content\n{}\n",
                        template, FILE_CONTENT_DELIMITER
                    ));
                }
                insert_string.push_str(&format!("{}\n", trimmed_line));
            }
        }
    }

    if insert_string.is_empty() {
        return Ok(primary_content);
    }

    insert_string.push_str(&format!("{}\n", FILE_CONTENT_DELIMITER));
    info!(
        "Caveman-like deduplication performed on the `{}` gitignore template, review the output",
        template
    );

    Ok(insert_string)
}

/// Lists the names of projects, tools, languages, ... from a locally cached gitignore template
/// repository.
fn list_templates(app_options: &mut Options) -> Result<(), Box<dyn StdErr>> {
    // FIXME: Review this function for a better approach if any.

    info!("Listing available templates");

    let mut template_list = String::new();
    let mut template_list_line_len = template_list.len();

    let template_paths = generate_template_paths(app_options)?;

    // NOTE: This sort is necessary to achieve a sorted list, unless the `BTreeMap`'s sort is
    // altered.
    let mut template_identifiers: Vec<String> = template_paths.keys().cloned().collect();
    template_identifiers.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

    // NOTE: This column print implementation yields the following average `time` results:
    // 0.03s user 0.01s system 99% cpu 0.047 total.
    // The former item count limited implementation yielded:
    // 0.01s user 0.00s system 96% cpu 0.011 total.
    let mut max_item_length = 0;
    for key in template_identifiers.iter() {
        let len = key.len();
        if len > max_item_length {
            max_item_length = len
        }
    }
    max_item_length += 1;
    debug!("Max list item length: {}", max_item_length);

    for key in template_identifiers.iter() {
        let mut key_string = key.to_string();
        for _ in key.len()..max_item_length {
            key_string.push_str(" ");
        }

        if template_list_line_len + max_item_length <= TEMPLATE_LIST_OUTPUT_LIMIT {
            template_list.push_str(&key_string);
            template_list_line_len += max_item_length
        } else {
            template_list.push_str(&format!("\n{}", key_string));
            template_list_line_len = max_item_length
        }
    }

    println!("{}", template_list);
    debug!("Done listing available templates");

    Ok(())
}

/// Generates [`TemplatePaths`] for the available gitignore template arguments supplied by a user.
///
/// This function generates a [`TemplatePaths`] item for the available gitignore template files
/// desired by a user.
/// Using the output of [`generate_template_paths`], the [`TemplatePaths`] is filtered to contain
/// entries explicitly requested by the user.
fn parse_templates(app_options: &mut Options) -> Result<TemplatePaths, Box<dyn StdErr>> {
    debug!("Parsing template options");

    let template_list = app_options.templates.clone();

    let mut available_templates = TemplatePaths::new();
    let template_paths = generate_template_paths(app_options)?;

    for template in template_list {
        // NOTE: The `clippy::option_map_unit_fn` warning was thrown for using a `map` on the
        // below operation.
        // Using `if let` is preferred for readability when a function doesn't return anything
        // meaningful: `std::unit`/`()`.
        if let Some(t_paths) = template_paths.get(&template) {
            *available_templates.entry(template).or_default() = t_paths.to_vec();
        };
    }

    debug!("Selected available template options");

    Ok(available_templates)
}

/// Updates the cached gitignore template repositories (git only).
///
/// This function fetches and merges the latest `HEAD` for an existing git repository, cloning one if
/// not locally cached.
/// This operation will not update a repository if it hasn't reached staleness (as defined by
/// [`const REPO_UPDATE_LIMIT`]) & the update operation isn't desired by the user.
///
/// REF: [github/nabijaczleweli](https://github.com/nabijaczleweli/cargo-update/blob/master/src/ops/mod.rs)
fn update_gitignore_repos(app_options: &mut Options) -> Result<(), Box<dyn StdErr>> {
    use git2::build::CheckoutBuilder;
    use std::time::SystemTime;

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
                debug!(
                    "Repository is cached locally, updating: {}",
                    repo_det.repo_path
                );

                repo.find_remote("origin")?.fetch(&["master"], None, None)?;
                let fetch_head = repo
                    .find_reference("FETCH_HEAD")?
                    .peel(git2::ObjectType::Any)?;
                repo.reset(&fetch_head, git2::ResetType::Hard, Some(&mut checkout))?;
            }
            Err(_) => {
                info!("Repository not cached locally: {}", repo_det.repo_path);
                clone_repository(app_options, &repo_det)?;
            }
        };

        info!("Updated gitignore repo: {}", repo_det.repo_path);
    }

    app_options.state.last_update = SystemTime::now();

    Ok(())
}

/// Clones a git repository into a local cache directory.
fn clone_repository(
    app_options: &Options,
    repo_det: &RepoDetails,
) -> Result<Repository, Box<dyn StdErr>> {
    use std::fs::DirBuilder;

    info!("Cloning gitignore repo: {}", repo_det.repo_path);

    let absolute_repo_path = absolute_repo_path!(app_options, repo_det);

    DirBuilder::new()
        .recursive(true)
        .create(&app_options.config.repo.repo_parent_dir)?;

    // NOTE: Wrapped in `Ok` to allow for the conversion of `git::error::Error` to `Box<dyn std::error::Error>`.
    Ok(Repository::clone_recurse(
        &repo_det.repo_url,
        &absolute_repo_path,
    )?)
}

/// Generates a [`TemplatePaths`] item.
///
/// This function prepares a [`TemplatePaths`] variable then calls [`update_template_paths`] to
/// update it.
fn generate_template_paths(app_options: &mut Options) -> Result<TemplatePaths, Box<dyn StdErr>> {
    let mut template_paths = TemplatePaths::new();

    for repo_det in app_options.config.repo.repo_dets.iter() {
        if repo_det.ignore {
            continue;
        }

        let absolute_repo_path = absolute_repo_path!(app_options, repo_det);

        // If the repository doesn't exist.
        if !Path::new(&absolute_repo_path).is_dir() {
            clone_repository(&app_options, &repo_det)?;
        };

        update_template_paths(&Path::new(&absolute_repo_path), &mut template_paths)?;
    }
    debug!("Template hash map: {:#?}", template_paths);

    Ok(template_paths)
}

/// Populates a [`TemplatePaths`] item with filepath entries.
///
/// This function recurses on the contents of the cached gitignore template repositories, appending
/// filepath entries to the passed [`TemplatePaths`] item for all available templates.
fn update_template_paths(dir: &Path, template_paths: &mut TemplatePaths) -> io::Result<()> {
    debug!("Updating template file paths for: {}", dir.display());

    // Store template name & path in hashmap.
    for entry in fs::read_dir(dir)? {
        let entry = entry?;

        if ignore_file(&entry) {
            continue;
        }

        let entry_path = entry.path();
        let entry_path_string = entry_path.clone().into_os_string().into_string().unwrap();

        if entry_path.is_dir() {
            update_template_paths(&entry_path, template_paths)?;
            debug!("Template scan directory: {}", &entry_path_string);

            continue;
        }

        let template = template_paths
            .entry(remove_filetype(&entry.path()))
            .or_default();

        template.push(entry_path_string);
    }

    debug!("Done updating template file paths for: {}", dir.display());

    Ok(())
}

/// Removes the filetype from a pathname.
///
/// This function calls [`std::path::Path`] operations to return a filename without the extension.
fn remove_filetype(path: &Path) -> String {
    path.file_stem()
        .unwrap()
        .to_os_string()
        .into_string()
        .unwrap()
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
