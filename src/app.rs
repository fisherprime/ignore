// SPDX-License-Identifier: MIT

extern crate git2;

/// `self::`` doesn't work here.
///
/// `super::` and `crate::` work.
/// Note, `super::` & `self::` are relative to the current module while `crate::` is relative to
/// the crate root.
use crate::config::Options;

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

/// Binary tree hash map type alias for simplicity.
type TemplatePaths = BTreeMap<String, Vec<String>>;

/// run handles the execution of ignore-ng's functions.
///
/// Using the parsed runtime config options, runs a task specified by ignore-ng's arguments then
/// overwrites the config file.
/// This function returns an error to the calling function on occurrence.
///
/// # Examples
///
/// ```
/// mod app;
///
/// use app::run;
///
/// if let Err(err) = run() {
///     panic!("Application error: {}", err)
/// }
/// ```
pub fn run(mut app_options: Options) -> Result<(), Box<dyn Error>> {
    if app_options.update_repo {
        update_gitignore_repo(&app_options)?;
    }

    if app_options.list_templates {
        list_templates(&mut app_options)?;
    }

    if app_options.generate_gitignore {
        generate_gitignore(&mut app_options)?;
    }

    app_options.save_config()?;

    Ok(())
}

/// generate_gitignore consolidates locally cached gitignore template files.
///
/// This function calls the template option parsing function then the template consolidation
/// function for the user defined gitignore template arguments, yielding a consolidated gitignore
/// file.
///
/// # Panics
///
/// This function will panic should reading the contents of a gitignore template fail.
///
/// # Examples
///
/// **Requires the user specify the `template` argument.**
///
/// ```
/// mod app;
/// mod config;
///
/// use app::generate_gitignore;
/// use config::Options;
///
/// if let Ok(mut opts) = Options::parse() {
///     generate_gitignore(&mut opts);
/// }
/// ```
fn generate_gitignore(app_options: &mut Options) -> Result<(), Box<dyn Error>> {
    info!("Generating gitignore");

    let mut consolidation_file: File;

    let consolidation_string: String;

    let available_templates =
        parse_templates(app_options).expect("Failed to parse the template argument");
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
    } else {
        warn!(
            "Specified template(s) could not be located (names are case sensitive): {:?}",
            app_options.templates
        );
    }

    Ok(())
}

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

                    template_file
                        .read_to_string(&mut temp_string)
                        .expect("Error reading template file");

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
                // TODO: replace with deduplication_logic.
                template_string += &temp_string;
                // TODO: end replacement deduplication_logic.
            }
        } else {
            template_string += &template_vec.pop().unwrap();
        }

        template_string += format!("{}\n", delimiter).as_str();
        consolidation_string += template_string.as_str();
    }

    Ok(consolidation_string)
}

fn list_templates(app_options: &mut Options) -> Result<(), Box<dyn Error>> {
    info!("Listing available templates");

    let list_width = 6;

    let mut list_string = String::new();

    let mut key_vector: Vec<String>;

    let template_paths = generate_template_paths(app_options)?;

    /* app_options.template_paths = match sort_template_paths(&app_options.template_paths) {
     *     Some(sort) => sort,
     *     None => panic!("Template file paths B-tree map not sorted"),
     * };
     * debug!("Sorted template hash: {:?}", app_options.template_paths); */

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

// Generate a B-tree map of available requested templates.
fn parse_templates(app_options: &mut Options) -> Result<TemplatePaths, Box<dyn Error>> {
    debug!("Parsing template options");

    let mut available_templates = TemplatePaths::new();

    let template_list = app_options.templates.clone();

    let template_paths = generate_template_paths(app_options)?;

    /* template_paths = match sort_template_paths(&template_paths) {
     *     Some(sort) => sort,
     *     None => panic!("Template file paths B-tree map not sorted"),
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

// REF: https://github.com/nabijaczleweli/cargo-update/blob/master/src/ops/mod.rs
fn update_gitignore_repo(app_options: &Options) -> Result<(), git2::Error> {
    info!("Updating gitignore repo(s)");

    let mut checkout = CheckoutBuilder::new();

    for repo_det in app_options.config.repo.repo_dets.iter() {
        /* let repo: Repository;
         * let fetch_head: Object; */

        let absolute_repo_path = format!(
            "{}/{}",
            app_options.config.repo.repo_parent_dir, repo_det.repo_path
        );

        let repo = Repository::discover(&absolute_repo_path).unwrap_or_else(|_| {
            info!(
                "Repository not cached locally, cloning: {}",
                repo_det.repo_path
            );

            let err_string = &format!(
                "Failed to clone: {} into: {:?}",
                repo_det.repo_url, repo_det.repo_path
            );

            DirBuilder::new()
                .recursive(true)
                .create(&app_options.config.repo.repo_parent_dir)
                .expect("Error creating repository cache directory hierarchy");

            Repository::clone_recurse(&repo_det.repo_url, &absolute_repo_path).expect(err_string)
        });

        debug!("Repository is available: {}", repo_det.repo_path);

        repo.find_remote("origin")
            .unwrap()
            .fetch(&["master"], None, None)?;

        let fetch_head = repo
            .find_reference("FETCH_HEAD")
            .unwrap()
            .peel(git2::ObjectType::Any)?;
        repo.reset(&fetch_head, git2::ResetType::Hard, Some(&mut checkout))?;

        info!("Updated gitignore repo: {}", repo_det.repo_path);
    }

    Ok(())
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

fn generate_template_paths(app_options: &mut Options) -> Result<TemplatePaths, Box<dyn Error>> {
    let mut template_paths = TemplatePaths::new();

    for repo_det in app_options.config.repo.repo_dets.iter() {
        if repo_det.ignore {
            continue;
        }

        let absolute_repo_path = format!(
            "{}/{}",
            app_options.config.repo.repo_parent_dir, repo_det.repo_path
        );

        if !Path::new(&absolute_repo_path).is_dir() {
            update_gitignore_repo(&app_options)?;
        };

        update_template_paths(&Path::new(&absolute_repo_path), &mut template_paths)?;
        debug!("Template hash: {:?}", template_paths);
    }

    Ok(template_paths)
}

fn update_template_paths(dir: &Path, template_paths: &mut TemplatePaths) -> io::Result<()> {
    debug!(
        "Updating template file paths, dir: {}",
        dir.as_os_str().to_str().unwrap()
    );

    // Store template name & path in hashmap.
    for entry in fs::read_dir(dir)? {
        let entry_path_string: String;

        let entry = entry?;

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

        // TODO: refine_filetype_removal.
        let t_filename = entry.file_name();
        #[allow(clippy::single_char_pattern)]
        let t_filename_split = t_filename
            .to_str()
            .unwrap()
            .split(".")
            .collect::<Vec<&str>>();
        let template = template_paths
            .entry(t_filename_split[0].to_string())
            .or_default();
        // TODO: end refine_filetype_removal.

        template.push(entry_path_string);
    }

    debug!(
        "Done updating template file paths, dir: {}",
        dir.as_os_str().to_str().unwrap()
    );

    Ok(())
}

fn is_hidden(entry: &DirEntry) -> bool {
    #[allow(clippy::single_char_pattern)]
    entry
        .file_name()
        .to_str()
        .map(|f_name| f_name.starts_with("."))
        .unwrap_or(false)
}

fn ignore_file(entry: &DirEntry) -> bool {
    // let ignores = Vec!["CHANGELOG", "LICENSE", "README", "CONTRIBUTING"];
    entry
        .file_name()
        .to_str()
        .map(|f_name| f_name.ends_with("md") || f_name.starts_with("LICENSE"))
        .unwrap_or(false)
        || is_hidden(&entry)
}
