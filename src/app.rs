// SPDX-License-Identifier: MIT

//! The `app` module defines user-executable tasks.

/* `self::`` doesn't work here.
 *
 * `super::` and `crate::` work.
 * Note: `super::` & `self::` are relative to the current module while `crate::` is relative to the
 * crate root.
 */
use crate::config::{runtime::Operation, runtime::RuntimeConfig};
use crate::errors::{Error, ErrorKind};
use crate::git::{fetch_repository, update_gitignore_repos};

use std::collections::btree_map::BTreeMap;
use std::error::Error as StdErr;
use std::fs::{self, DirEntry, File};
use std::io::{self, prelude::*};
use std::path::Path;
use std::time::SystemTime;

use regex::Regex;

/// `Binary tree hash-map` alias for simplicity.
type TemplatePaths = BTreeMap<String, Vec<String>>;

/// Macro used to reduce repetition when defining a cached repository's absolute path.
#[macro_export]
macro_rules! absolute_repo_path {
    ($parent:expr, $base:expr) => {
        format!("{}/{}", $parent.config.repository.cache_dir, $base.path)
    };
}

/// Const specifying the column limit to wrap an [`Operation::ListAvailableTemplates`] list line.
const TEMPLATE_LIST_OUTPUT_LIMIT: usize = 100;

/// Const specifying the file content delimiter used.
const FILE_CONTENT_DELIMITER: &str = "# ----";

/// Const specifying the delimiter for supplementary template content
const TEMPLATE_SUPPLEMENT_DELIMITER: &str = "# ****";

lazy_static! {
    static ref GITIGNORE_ENTRY_REGEX: Regex =
        Regex::new(r"[\*/!]").expect("failed to compile gitignore entry regex");
}

/// Handles the execution of `ignore`'s functions.
///
/// Using the parsed [`RuntimeConfig`], this function runs a task specified by the user in `ignore`'s
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
/// use crate::config::RuntimeConfig;
///
/// RuntimeConfig::parse().map(|opts| {
///     run(opts)
///         .unwrap_or_else(|err| panic!("app: failed with: {}", err))
/// })
/// ```
pub fn run(mut app_confg: RuntimeConfig) -> Result<(), Box<dyn StdErr>> {
    if app_confg.state.check_staleness(&SystemTime::now())? {
        update_gitignore_repos(&mut app_confg);
        if app_confg.operation == Operation::UpdateRepositories {
            return app_confg.state.save_to_file();
        }
    }

    match app_confg.operation {
        Operation::GenerateGitignore => generate_gitignore(&mut app_confg)?,
        Operation::ListAvailableTemplates => list_templates(&mut app_confg)?,
        Operation::UpdateRepositories => update_gitignore_repos(&mut app_confg),
        Operation::GenerateCompletions => app_confg.generate_completions()?,
        Operation::Else => info!("app: no operation specified, this shouldn't have happened"),
    }

    app_confg.state.save_to_file()
}

/// Consolidates locally cached gitignore template(s).
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
/// use config::RuntimeConfig;
///
/// RuntimeConfig::parse()
///     .map(|app_conf| generate_gitignore(&mut app_conf))
///     .unwrap_or_else(|err| panic!("app: failed with: {}", err))
///
/// ```
fn generate_gitignore(app_confg: &mut RuntimeConfig) -> Result<(), Box<dyn StdErr>> {
    use std::fs::OpenOptions;

    info!("app: generating gitignore");

    let available_templates = parse_templates(app_confg)?;
    debug!("app: available templates {:#?}", available_templates);

    let mut consolidation_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&app_confg.gitignore_output_file)?;
    debug!("app: opened gitignore template consolidation file");

    consolidation_file.set_len(0)?;

    let consolidation_string = concatenate_templates(&app_confg.templates, available_templates)?;
    consolidation_file.write_all(consolidation_string.as_bytes())?;
    info!(
        "app: generated gitignore {}",
        app_confg.gitignore_output_file
    );

    Ok(())
}

/// Concatenates gitignore template(s) specified by the user.
///
/// This function acts on a [`TemplatePaths`] item for the template arguments specified by a user,
/// consolidating the file paths listed within the item.
fn concatenate_templates(
    requested_templates: &[String],
    available_templates: TemplatePaths,
) -> Result<String, Box<dyn StdErr>> {
    let mut consolidation_string = String::new();
    let mut return_string = String::new();
    let mut templates_used = String::new();

    if available_templates.is_empty() {
        warn!(
            "app: could not locate template(s) (names are case sensitive) {:?}",
            requested_templates
        );
        return Err(Box::new(Error::from(ErrorKind::MissingTemplates)));
    }

    // Iterate over template_paths, opening necessary file & concatenating them.
    for (template, file_paths) in available_templates {
        let file_paths = &file_paths;

        let mut template_string = format!("\n# {}\n{}\n", template, FILE_CONTENT_DELIMITER);

        let mut template_vec = Vec::<String>::new();

        for file_path in file_paths {
            debug!("parsing: {}", file_path);
            match File::open(file_path) {
                Ok(mut template_file) => {
                    let mut buffer = String::new();

                    template_file.read_to_string(&mut buffer)?;
                    template_vec.push(buffer.to_owned());

                    debug!(
                        "app: appended {} content to {} template vector",
                        file_path, template
                    );
                }
                Err(err) => {
                    error!("app: failed to open gitignore template file {}", err);
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
            "app: could not use template(s) (names are case sensitive) {:?}",
            requested_templates
        );
        return Err(Box::new(Error::from(ErrorKind::MissingTemplates)));
    }

    return_string.push_str("#\n# .gitignore\n#\n\n");
    return_string.push_str(&format!(
        "# Templates used:{}\n{}",
        templates_used, consolidation_string
    ));

    Ok(return_string)
}

/// Deduplicates gitignore template content.
fn dedup_templates(template: &str, template_vec: &mut [String]) -> Result<String, Box<dyn StdErr>> {
    // FIXME: Review this function for a better approach if any.
    // Iterating over all the lines for subsequent template files of a given technology seems
    // wasteful, they shouldn't be more than one so...

    info!(
        "app: deduplicating gitignore template entries for {}",
        template
    );

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
            }

            if insert_string.is_empty() {
                insert_string.push_str(&format!("{}\n", primary_content));
                insert_string.push_str(&format!(
                    "# {} supplementary content\n{}\n",
                    template, TEMPLATE_SUPPLEMENT_DELIMITER
                ));
            }
            insert_string.push_str(&format!("{}\n", trimmed_line));
        }
    }

    if insert_string.is_empty() {
        return Ok(primary_content);
    }

    insert_string.push_str(&format!("{}\n", TEMPLATE_SUPPLEMENT_DELIMITER));
    info!(
        "app: `{}` gitignore templates deduplicated, review the output",
        template
    );

    Ok(insert_string)
}

/// Lists the names of projects, tools, languages,… from locally cached gitignore template
/// repositories.
fn list_templates(app_conf: &mut RuntimeConfig) -> Result<(), Box<dyn StdErr>> {
    // FIXME: Review this function for a better approach if any.

    info!("app: listing available templates");

    let mut template_list = String::new();
    let mut template_list_line_len = template_list.len();

    let template_paths = generate_template_paths(app_conf)?;

    // NOTE: This sort is necessary to achieve a sorted list, unless the `BTreeMap`'s sort is
    // altered.
    let mut template_identifiers: Vec<_> = template_paths.keys().cloned().collect();
    template_identifiers.sort_by_key(|a| a.to_lowercase());

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
    debug!("app: max list item length {}", max_item_length);

    for key in template_identifiers.iter() {
        let mut key_string = key.to_string();
        for _ in key.len()..max_item_length {
            key_string.push(' ');
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
    debug!("app: done listing available templates");

    Ok(())
}

/// Generates [`TemplatePaths`] for the available gitignore template arguments supplied by a user.
///
/// This function generates a [`TemplatePaths`] item for the available gitignore template files
/// desired by a user.
/// Using the output of [`generate_template_paths`], the [`TemplatePaths`] is filtered to contain
/// entries explicitly requested by the user.
fn parse_templates(app_conf: &mut RuntimeConfig) -> Result<TemplatePaths, Box<dyn StdErr>> {
    debug!("app: parsing template options");

    let template_list = app_conf.templates.clone();

    let mut available_templates = TemplatePaths::new();
    let template_paths = generate_template_paths(app_conf)?;

    for template in template_list {
        // NOTE: The `clippy::option_map_unit_fn` warning was thrown for using a `map` on the below
        // operation.
        //
        // Using `if let` is preferred for readability when a function doesn't return anything
        // meaningful: `std::unit`/`()`.
        if let Some(t_paths) = template_paths.get(&template) {
            *available_templates.entry(template).or_default() = t_paths.to_vec();
        };
    }

    debug!("app: selected available template options");

    Ok(available_templates)
}

/// Populates a [`TemplatePaths`] item with filepath entries.
///
/// This function recurses on the content of the cached gitignore template repositories, appending
/// filepath entries to the passed [`TemplatePaths`] item for all available templates.
fn update_template_paths(dir: &Path, template_paths: &mut TemplatePaths) -> io::Result<()> {
    debug!("app: updating template file paths for {}", dir.display());

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
            debug!("app: template scan directory {}", &entry_path_string);

            continue;
        }

        let template = template_paths
            .entry(remove_filetype(&entry.path()))
            .or_default();

        template.push(entry_path_string);
    }

    debug!(
        "app: done updating template file paths for {}",
        dir.display()
    );

    Ok(())
}

/// Generates a [`TemplatePaths`] item.
///
/// This function prepares a [`TemplatePaths`] variable then calls [`update_template_paths`] to
/// update it.
fn generate_template_paths(app_conf: &mut RuntimeConfig) -> Result<TemplatePaths, Box<dyn StdErr>> {
    let mut template_paths = TemplatePaths::new();

    for conf in app_conf.config.repository.config.iter() {
        if conf.skip {
            continue;
        }

        let absolute_repo_path = absolute_repo_path!(app_conf, conf);

        // If the repository doesn't exist.
        if !Path::new(&absolute_repo_path).is_dir() {
            // And the repository is not a repository.
            if !conf.url.is_empty() {
                fetch_repository(app_conf, conf)?;
            }
        };

        update_template_paths(Path::new(&absolute_repo_path), &mut template_paths)?;
    }
    debug!("app: template hash map {:#?}", template_paths);

    Ok(template_paths)
}

/// Removes the file type from a pathname.
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
        || is_hidden(entry)
}
