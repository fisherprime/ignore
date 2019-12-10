// SPDX-License-Identifier: MIT

extern crate git2;

use crate::config::Options;

use git2::build::CheckoutBuilder;
use git2::{Object, Repository};
use std::collections::btree_map::BTreeMap;
// use std::collections::hash_map::HashMap;
use std::error::Error;
use std::fs::{self, DirBuilder, DirEntry, File, OpenOptions};
use std::io;
use std::io::prelude::*;
use std::path::Path;

pub fn run(mut app_options: Options) -> Result<(), Box<dyn Error>> {
    if app_options.update_repo {
        update_gitignore_repo(&app_options)?;
    }

    if app_options.list_templates {
        list_templates(&mut app_options)?
    }

    if app_options.generate_gitignore {
        generate_gitignore(&mut app_options)?;
    }

    app_options.save();
    Ok(())
}

pub fn generate_gitignore(app_options: &mut Options) -> Result<(), io::Error> {
    info!("Generating gitignore");

    let delimiter = "# ----";
    let available_templates: BTreeMap<String, Vec<String>>;

    let mut consolidation_file: File;

    let mut consolidation_string = "#\n# .gitignore\n#\n".to_string();

    consolidation_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&app_options.output_file)?;
    // .expect("Error opening gitignore consolidation file");
    debug!("Opened gitignore consolidation file");

    available_templates =
        parse_templates(app_options).expect("Failed to parse the template argument");
    debug!("Available templates: {:?}", available_templates);

    if available_templates.is_empty() {
        warn!("Specified template(s) could not be located; names are case sensitive");
        return Ok(());
    }

    // Iterate over template_paths, opening necessary file & concatenating them
    for (template, file_paths) in available_templates {
        let file_paths = &file_paths;

        for file_path in file_paths {
            debug!("Parsing: {}", file_path);

            let mut template_string = String::new();

            let mut template_file = File::open(file_path).unwrap_or_else(|err| {
                // Prefer to break out of the loop
                panic!("Error opening gitignore template file: {:?}", err);
                // error!("Error opening gitignore template file: {:?}", err);
            });

            template_file
                .read_to_string(&mut template_string)
                .expect("Error reading template file");
            consolidation_string += format!("\n# {}\n", template).as_str();
            consolidation_string +=
                format!("{}\n{}{}\n", delimiter, template_string, delimiter).as_str();
            debug!("Written {} to consolidation string", file_path);
        }
    }

    consolidation_file.set_len(0)?;
    // .expect("Error truncating consolidation file");
    consolidation_file.write_all(consolidation_string.as_bytes())?;
    // .expect("Error writing to gitignore consolidation file");
    info!("Done generating gitignore: {}", app_options.output_file);

    Ok(())
}

pub fn list_templates(app_options: &mut Options) -> Result<(), Box<dyn Error>> {
    info!("Listing available templates");

    let list_width = 6;

    let absolute_repo_path: String;
    let mut list_string = String::new();

    let mut key_vector: Vec<String>;

    absolute_repo_path = format!(
        "{}/{}",
        app_options.config.repo.repo_parent_dir, app_options.config.repo.repo_path
    );

    update_template_paths(
        &Path::new(&absolute_repo_path),
        &mut app_options.template_paths,
    )
    .expect("Error updating template file paths");
    debug!("Template hash: {:?}", app_options.template_paths);

    /* app_options.template_paths = match sort_template_paths(&app_options.template_paths) {
     *     Some(sort) => sort,
     *     None => panic!("Template file paths B-tree map not sorted"),
     * };
     * debug!("Sorted template hash: {:?}", app_options.template_paths); */

    key_vector = app_options.template_paths.keys().cloned().collect();
    key_vector.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

    for (index, key) in key_vector.iter().enumerate() {
        list_string += key;
        list_string += " ";

        if index % list_width == 0 {
            list_string += "\n";
        }
    }
    println!("{}", list_string);

    info!("Done listing available templates");

    Ok(())
}

// Generate a B-tree map of available requested templates
fn parse_templates(app_options: &mut Options) -> Result<TemplatePaths, Box<dyn Error>> {
    debug!("Parsing template options");

    let absolute_repo_path: String;

    let mut available_templates = BTreeMap::<String, Vec<String>>::new();
    let mut template_paths = BTreeMap::<String, Vec<String>>::new();

    let template_list = app_options.templates.clone();

    absolute_repo_path = format!(
        "{}/{}",
        app_options.config.repo.repo_parent_dir, app_options.config.repo.repo_path
    );

    update_template_paths(&Path::new(&absolute_repo_path), &mut template_paths)
        .expect("Error updating template file paths");
    debug!("Template path B-tree map updated");
    debug!("Template hash: {:?}", template_paths);
    debug!("Template hash: {:?}", template_paths);

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
pub fn update_gitignore_repo(app_options: &Options) -> Result<(), git2::Error> {
    info!("Updating gitignore repo");

    let absolute_repo_path: String;

    let repo: Repository;

    let fetch_head: Object;

    let mut checkout = CheckoutBuilder::new();

    absolute_repo_path = format!(
        "{}/{}",
        app_options.config.repo.repo_parent_dir, app_options.config.repo.repo_path
    );

    repo = Repository::discover(&absolute_repo_path).unwrap_or_else(|_| {
        info!("Repository not cached locally, cloning");

        let err_string = &format!(
            "Failed to clone: {} into: {:?}",
            app_options.config.repo.repo_url, app_options.config.repo.repo_path
        );

        DirBuilder::new()
            .recursive(true)
            .create(&app_options.config.repo.repo_parent_dir)
            .expect("Error creating repository cache directory hierarchy");

        Repository::clone_recurse(&app_options.config.repo.repo_url, &absolute_repo_path)
            .expect(err_string)
    });

    debug!("Repository is available");


        debug!("Repository is available: {}", repo_det.repo_path);

        repo.find_remote("origin")
            .unwrap()
            .fetch(&["master"], None, None)?;
        // .expect("Failed to fetch remote repo");

        let fetch_head = repo
            .find_reference("FETCH_HEAD")
            .unwrap()
            .peel(git2::ObjectType::Any)?;
        // .expect("Error peeling object from FETCH_HEAD reference");
        repo.reset(&fetch_head, git2::ResetType::Hard, Some(&mut checkout))?;
        // .expect("Error resetting repo head to FETCH_HEAD");

        info!("Updated gitignore repo: {}", repo_det.repo_path);
    }

    Ok(())
}

// Using a BTreeMap is faster
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

fn update_template_paths(
    dir: &Path,
    template_paths: &mut BTreeMap<String, Vec<String>>,
) -> io::Result<()> {
    debug!(
        "Updating template file paths, dir: {}",
        dir.as_os_str().to_str().unwrap()
    );

    // Store template name & path in hashmap
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

        // TODO: refine filetype removal
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
