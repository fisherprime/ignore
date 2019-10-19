// SPDX-License-Identifier: MIT

extern crate chrono;
extern crate dirs;
extern crate fern;
extern crate regex;
extern crate serde;
extern crate toml;

// use regex::Regex;
use clap::{App, Arg, ArgMatches};
use serde::{Deserialize, Serialize};
use std::collections::btree_map::BTreeMap;
use std::ffi::OsString;
use std::fs::{DirBuilder, File, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

const REPO_UPDATE_LIMIT: u64 = 60 * 60 * 24 * 7;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub core: CoreConfig,
    pub repo: RepoConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CoreConfig {
    pub last_run: SystemTime,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RepoConfig {
    pub repo_parent_dir: String,
    pub repo_path: String,
    pub repo_url: String,
}

#[derive(Debug)]
pub struct Options {
    pub generate_gitignore: bool,
    pub list_templates: bool,
    pub update_repo: bool,

    pub config_path: String,
    pub output_file: String,

    pub templates: Vec<String>,

    pub template_paths: BTreeMap<String, String>,
}

impl Config {
    pub fn parse() -> Option<(Config, Options)> {
        debug!("Parsing command arguments & config file");

        let mut app_config: Config;
        let app_options: Options;

        let matches: ArgMatches;

        let default_gitignore_repo: String = "https://github.com/github/gitignore".to_string();
        let r_path: String;

        let mut r_parent_dir: PathBuf;

        let now = SystemTime::now();

        // env!("CARGO_PKG_VERSION")
        // Doesn't live long enough
        matches = App::new("ignore-ng")
        .version(crate_version!())
        .about("Generated .gitignore files")
        .author("fisherprime")
        .arg(
            Arg::with_name("list")
                .short("l")
                .long("list")
                .help("List all available languages, tools & projects."),
        )
        .arg(
            Arg::with_name("template")
                .short("t")
                .long("templates")
                .value_name("TEMPLATE")
                .takes_value(true)
                .multiple(true)
                .help(
                "Case sensitive specification of language(s), tool(s) and/or project template(s) to use in generating .gitignore."),
        )
        .arg(
            Arg::with_name("output")
            .short("o")
            .long("output")
            .value_name("FILE")
            .takes_value(true)
            .help("Specify output filename, defaults to: gitignore-ng."),
            )
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .takes_value(true)
            .help("Specify alternative config file to use."))
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Set the level of verbosity for logs: -v, -vv."),
        )
        .get_matches();
        debug!("Done parsing command flags");

        setup_logger(&matches).expect("Error setting up logger");

        /*
         *     let re = Regex::new(URL_PREFIX_REGEX)
         *         .unwrap()
         *         .replace(default_gitignore_repo, ""); */

        // TODO: fix, messy section
        let gitignore_repo_split: Vec<&str> = default_gitignore_repo.split('/').collect();
        let gitignore_split_len = gitignore_repo_split.len();

        r_path = format!(
            "{}/{}",
            gitignore_repo_split[gitignore_split_len - 2],
            gitignore_repo_split[gitignore_split_len - 1]
        );
        r_parent_dir = dirs::cache_dir().expect("Error obtaining system's cache directory");
        r_parent_dir.push("ignore-ng/repos");

        app_config = Config {
            core: CoreConfig {
                // Sort out duration since error
                last_run: now - Duration::new(0, 500),
            },
            repo: RepoConfig {
                repo_url: default_gitignore_repo,
                repo_parent_dir: r_parent_dir.into_os_string().into_string().unwrap(),
                repo_path: r_path,
            },
        };

        app_options = Options {
            generate_gitignore: matches.is_present("template"),
            list_templates: matches.is_present("list"),
            update_repo: (now.duration_since(app_config.core.last_run).unwrap()
                > Duration::new(REPO_UPDATE_LIMIT, 0))
                || (now.duration_since(app_config.core.last_run).unwrap() == Duration::new(0, 500)),
            config_path: "".to_string(),
            output_file: matches
                .value_of("output")
                .unwrap_or("gitignore-ng")
                .to_string(),
            templates: match matches.values_of("template") {
                Some(templates_vec) => {
                    let mut temp_string_vec: Vec<String> = Vec::new();
                    let temp_str_vec = templates_vec.collect::<Vec<&str>>();

                    for template in temp_str_vec {
                        temp_string_vec.push(template.to_string());
                    }

                    temp_string_vec
                }
                None => ["".to_string()].to_vec(),
            },
            template_paths: BTreeMap::<String, String>::new(),
        };

        if let Some(cfg) = app_config.parse_config_file(&matches) {
            app_config = cfg;
        }

        debug!("Done parsing command arguments & config file");
        debug!("Config: {:?}", app_config);
        debug!("Options: {:?}", app_options);

        Some((app_config, app_options))
    }

    // Passing a reference to avoid taking ownership
    fn parse_config_file(&self, matches: &ArgMatches) -> Option<Config> {
        debug!("Parsing config file");

        let config: Config;

        let mut config_string = String::new();
        let mut config_file_path = String::new();

        let mut default_config_file: PathBuf;

        let def_cfg_os_str: OsString;

        let mut config_file: File;

        let read_bytes: usize;

        default_config_file =
            dirs::config_dir().expect("Error obtaining system's config directory");
        default_config_file.push("ignore-ng/config.toml");

        if let Some(path) = matches.value_of("config") {
            config_file_path = path.to_string();

            debug!("Using user supplied config file path");
        } else {
            def_cfg_os_str = default_config_file.clone().into_os_string();

            if let Some(cfg_str) = def_cfg_os_str.to_str() {
                config_file_path = cfg_str.to_string();
            }

            debug!("Using default config file path");
        }

        config_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&config_file_path)
            .unwrap_or_else(|err| {
                if err.kind() == ErrorKind::NotFound {
                    if config_file_path.eq(default_config_file.into_os_string().to_str().unwrap()) {
                        // Create config directory
                        if let Some(conf_dir) = Path::new(&config_file_path).parent() {
                            if !conf_dir.is_dir() {
                                DirBuilder::new()
                                    .recursive(true)
                                    .create(conf_dir)
                                    .expect("Error creating config file directory hierarchy");
                            }
                        }

                        File::create(config_file_path).expect("Error creating default config file")
                    } else {
                        panic!("Could not find config file: {:?}", err);
                    }
                } else {
                    panic!("Could not open config file: {:?}", err);
                }
            });

        read_bytes = config_file
            .read_to_string(&mut config_string)
            .unwrap_or_else(|err| match err.kind() {
                ErrorKind::NotFound => {
                    error!("No config file: {}", err);
                    0
                }
                ErrorKind::Other => {
                    error!("Other config file err: {}", err);
                    0
                }
                _ => {
                    debug!("Read config file: No error");
                    0
                }
            });

        if read_bytes > 0 {
            // Temporary value dropped
            config = toml::from_str(config_string.trim()).unwrap();
            debug!("Done parsing config file");

            return Some(config);
        }

        info!("Config file is empty, using default config values");

        // Write default config to file
        config_file
            .write_all(
                toml::to_string(&self)
                    .expect("Error writing to config file")
                    .as_bytes(),
            )
            .unwrap();
        debug!("Updated config file with config values");

        debug!("Done parsing config file");

        None
    }

    #[allow(dead_code)]
    pub fn update_config_file(self, config_file_path: &str) {
        let mut config_file: File;

        debug!("Config file path: {}", config_file_path);

        config_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(config_file_path)
            .unwrap();

        config_file
            .write_all(
                toml::to_string(&self)
                    .expect("Error writing to config file")
                    .as_bytes(),
            )
            .unwrap();
        debug!("Updated config file with config values");
    }
}

// REF: https://mathiasbynens.be/demo/url-regex
// TODO: validate regex
/* const URL_PREFIX_REGEX: &str = */
/* r"#(?i)\b((?:[a-z][\w-]+:(?:/{1,3}|[a-z0-9%])|www\d{0,3}[.]|[a-z0-9.\-]+[.][a-z]{2,4}/))"; */

fn setup_logger(matches: &ArgMatches) -> Result<(), fern::InitError> {
    debug!("Setting up logger");

    let log_max_level = match matches.occurrences_of("verbosity") {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        2 => log::LevelFilter::Trace,
        _ => log::LevelFilter::Off,
    };

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log_max_level)
        .chain(std::io::stdout())
        // .chain(fern::log_file("output.log")?)
        .apply()?;

    debug!("Done setting up logger");

    Ok(())
}

/* #[cfg(test)]
 * mod tests {
 *     use super::*;
 *
 *     #[test]
 *     fn setup_logger_test() {
 *         assert!(asdasda)
 *     }
 * } */
