// SPDX-License-Identifier: MIT

extern crate chrono;
extern crate dirs;
extern crate fern;
extern crate serde;
extern crate toml;

// use std::collections::btree_map::BTreeMap;
// use std::collections::hash_map::HashMap;
// use std::ffi::OsString;
use clap::{App, AppSettings, Arg, ArgMatches};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{DirBuilder, File, OpenOptions};
use std::io::prelude::*;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

const REPO_UPDATE_LIMIT: u64 = 60 * 60 * 24 * 7;

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Config {
    // Binary specific configuration options
    pub core: CoreConfig,

    // Repository specific configuration options
    pub repo: RepoConfig,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct CoreConfig {
    // Timestamp of the last time the binary was run
    pub last_run: SystemTime,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct RepoConfig {
    // Directory containing gitignore repositories
    pub repo_parent_dir: String,

    // Details for multiple/single template repository
    pub repo_dets: Vec<RepoDetails>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct RepoDetails {
    // Choice for ignoring repository usage for any task
    pub ignore: bool,

    // Relative path (to repo_parent_dir) of gitignore template repo to use
    pub repo_path: String,

    // URL of git repository containing gitignore templates
    pub repo_url: String,
}

#[derive(Debug)]
pub struct Options {
    // Config read from file
    pub config: Config,

    // Option to generate gitignore file
    pub generate_gitignore: bool,

    // Option to list available templates
    pub list_templates: bool,

    // Option to update repository
    pub update_repo: bool,

    // Path to configuration file
    pub config_path: String,

    // Path to output generated gitignore
    pub output_file: String,

    // List of templates user desires to use in gitignore generation
    pub templates: Vec<String>,
    // B-Tree hash map of all available template paths
    // pub template_paths: BTreeMap<String, Vec<String>>,
}

impl Config {
    pub fn new() -> Config {
        let default_gitignore_repo: String = "https://github.com/github/gitignore".to_string();
        let r_path: String;

        let mut r_parent_dir: PathBuf;

        let now = SystemTime::now();

        // TODO: fix, messy section
        // Get repo_path as defined in the Options struct
        let gitignore_repo_split: Vec<&str> = default_gitignore_repo.split('/').collect();
        let gitignore_split_len = gitignore_repo_split.len();

        r_path = format!(
            "{}/{}",
            gitignore_repo_split[gitignore_split_len - 2],
            gitignore_repo_split[gitignore_split_len - 1]
        );
        r_parent_dir = dirs::cache_dir().expect("Error obtaining system's cache directory");
        r_parent_dir.push("ignore-ng/repos");

        Config {
            core: CoreConfig {
                // Sort out duration since error
                last_run: now - Duration::new(0, 500),
            },
            repo: RepoConfig {
                repo_parent_dir: r_parent_dir.into_os_string().into_string().unwrap(),
                repo_dets: vec![RepoDetails {
                    ignore: false,
                    repo_url: default_gitignore_repo,
                    repo_path: r_path,
                }],
            },
        }
    }

    // Parse config file contents
    // Passing a reference to avoid taking ownership
    fn parse(&self, config_file_path: &str) -> Result<Config, Box<dyn Error>> {
        debug!("Parsing config file");

        let config: Config;

        let read_bytes: usize;

        let mut config_string = String::new();

        let mut default_config_file: PathBuf;

        let mut config_file: File;

        default_config_file =
            dirs::config_dir().expect("Error obtaining system's config directory");
        default_config_file.push("ignore-ng/config.toml");

        config_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(config_file_path)
            .unwrap_or_else(|err| {
                if err.kind() == ErrorKind::NotFound {
                    if config_file_path.eq(default_config_file.into_os_string().to_str().unwrap()) {
                        // Create default config directory
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
            .unwrap_or_else(|err| {
                if err.kind() == ErrorKind::NotFound {
                    error!("No config file: {}", err);
                } else {
                    error!("Error reading config file err: {}", err);
                }

                0
            });
        if read_bytes > 0 {
            // If config file isn't empty
            match toml::from_str(config_string.trim()) {
                Ok(cfg) => {
                    debug!("Done parsing config file");
                    return Ok(cfg);
                }
                Err(_) => {
                    info!("Backing up config file");
                    std::fs::copy(config_file_path, format!("{}.bak", config_file_path))
                        .expect("Could not backup config file");
                }
            }
        }

        info!("Config file is empty, using default config values");
        config = self.clone();

        // Write default config to file
        config_file.write_all(toml::to_string(&self)?.as_bytes())?;
        debug!("Updated config file with config values");

        Ok(config)
    }
}

impl Options {
    // Parse command arguments
    pub fn parse() -> Result<Options, Box<dyn Error>> {
        debug!("Parsing command arguments & config file");

        let mut config_file_path = String::new();

        let mut default_config_file: PathBuf;

        let app_config = Config::new();
        let app_options: Options;

        let matches: ArgMatches;

        let now = SystemTime::now();

        // env!("CARGO_PKG_VERSION")
        matches = App::new("ignore-ng")
            .setting(AppSettings::ArgRequiredElseHelp)
            .version(crate_version!())
            .about("Generated .gitignore files")
            .author("fisherprime")
            .arg(
                Arg::with_name("config")
                .help("Specify alternative config file to use.")
                .short("c")
                .long("config")
                .value_name("FILE")
                .takes_value(true)
            )
            .arg(
                Arg::with_name("list")
                .help("List all available languages, tools & projects.")
                .short("l")
                .long("list")
            )
            .arg(
                Arg::with_name("output")
                .help("Specify output filename, defaults to: gitignore-ng.")
                .short("o")
                .long("output")
                .value_name("FILE")
                .takes_value(true)
            )
            .arg(
                Arg::with_name("template")
                .help("Case sensitive specification of language(s), tool(s) and/or project template(s) to use in generating .gitignore.")
                .short("t")
                .long("templates")
                .value_name("TEMPLATE")
                .takes_value(true)
                .multiple(true)
            )
            .arg(
                Arg::with_name("update")
                .help("Manually update the gitignore template repo(s)")
                .short("u").long("update")
            )
            .arg(
                Arg::with_name("verbosity")
                .help("Set the level of verbosity for logs: -v, -vv.")
                .short("v")
                .long("verbose")
                .multiple(true)
            )
            .get_matches();
        debug!("Parsed command flags");

        setup_logger(&matches)?;

        default_config_file =
            dirs::config_dir().expect("Error obtaining system's config directory");
        default_config_file.push("ignore-ng/config.toml");

        if let Some(path) = matches.value_of("config") {
            config_file_path = path.to_string();

            debug!("Using user supplied config file path");
        } else {
            let def_cfg_os_str = default_config_file.into_os_string();

            if let Some(cfg_str) = def_cfg_os_str.to_str() {
                config_file_path = cfg_str.to_string();
            }

            debug!("Using default config file path");
        }

        /* // Create repo_path from repo_url
         * let re = Regex::new(URL_PREFIX_REGEX)
         *     .unwrap()
         *     .replace(default_gitignore_repo, ""); */

        app_options = Options {
            config: match app_config.parse(&config_file_path) {
                Ok(cfg) => cfg,
                Err(err) => {
                    error!("Config parse error, using the default: {}", err);

                    app_config.clone()
                }
            },
            generate_gitignore: matches.is_present("template"),
            list_templates: matches.is_present("list"),
            update_repo: if matches.is_present("update") {
                true
            } else {
                (now.duration_since(app_config.core.last_run)?
                    > Duration::new(REPO_UPDATE_LIMIT, 0))
                    || (now.duration_since(app_config.core.last_run)? == Duration::new(0, 500))
            },
            config_path: config_file_path,
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
            // template_paths: BTreeMap::<String, Vec<String>>::new(),
        };
        debug!(
            "Parsed command arguments & config file, options: {:?}",
            app_options
        );

        Ok(app_options)
    }

    pub fn save(self) {
        let mut config_file: File;

        debug!("Updating config in file path: {}", self.config_path);

        config_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(self.config_path)
            .unwrap();

        config_file
            .set_len(0)
            .expect("Error truncating config file");
        config_file
            .write_all(
                toml::to_string(&self.config)
                    .expect("Error writing to config file")
                    .as_bytes(),
            )
            .expect("Error writing to config file");
        debug!("Updated config file");
    }
}

// REF: https://mathiasbynens.be/demo/url-regex
// TODO: validate regex
/* const URL_PREFIX_REGEX: &str = */
/* r"#(?i)\b((?:[a-z][\w-]+:(?:/{1,3}|[a-z0-9%])|www\d{0,3}[.]|[a-z0-9.\-]+[.][a-z]{2,4}/))"; */

fn setup_logger(matches: &ArgMatches) -> Result<(), fern::InitError> {
    debug!("Setting up logger");

    let mut verbose = true;

    let log_max_level = match matches.occurrences_of("verbosity") {
        0 => {
            verbose = false;
            log::LevelFilter::Info
        }
        1 => log::LevelFilter::Debug,
        2 => log::LevelFilter::Trace,
        _ => {
            verbose = false;
            log::LevelFilter::Off
        }
    };

    if verbose {
        fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "{}[{}][{}] {}",
                    chrono::Local::now().format("[%Y-%m-%dT%H:%M:%S%z]"),
                    record.target(),
                    record.level(),
                    message
                ))
            })
            .level(log_max_level)
            .chain(std::io::stdout())
            // .chain(fern::log_file("output.log")?)
            .apply()?;
    } else {
        fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!("[{}] {}", record.level(), message))
            })
            .level(log_max_level)
            .chain(std::io::stdout())
            // .chain(fern::log_file("output.log")?)
            .apply()?;
    }

    debug!("Done setting up logger");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /**
     * Assert correctness of the default runtime options, includes the config
     * TODO: add necessary fields
     */
    /*     #[test]
     *     fn option_parse_test() {
     *         let options = match Options::parse() {
     *             Some(val) => val,
     *             None => None,
     *         };
     *
     *         assert!(options);
     *     } */

    /**
     * Assert correctness of the default config options
     */
    #[test]
    fn config_create_test() {
        let config = Config::new();

        let now = SystemTime::now();

        let mut parent_dir = dirs::cache_dir().unwrap();
        parent_dir.push("ignore-ng/repos");

        let hardcode_config = Config {
            core: CoreConfig {
                last_run: now - Duration::new(0, 500),
            },
            repo: RepoConfig {
                repo_parent_dir: parent_dir.into_os_string().into_string().unwrap(),
                repo_dets: vec![RepoDetails {
                    ignore: false,
                    repo_url: "https://github.com/github/gitignore".to_string(),
                    repo_path: "github/gitignore".to_string(),
                }],
            },
        };

        assert!(hardcode_config.repo.eq(&config.repo));
    }

    /**
     * Assert correctness of parsed config file.
     * Should test run only if file exists?
     */
    #[test]
    fn config_file_parse_test() {
        let mut config = Config::new();

        let mut config_path = dirs::config_dir().unwrap();
        config_path.push("ignore-ng/config.toml");

        // Parse probably empty config and populate it with current config
        config = match config.parse(&config_path.clone().into_os_string().into_string().unwrap()) {
            Ok(cfg) => cfg,
            Err(err) => {
                error!("Config parse error, using the default: {}", err);

                config.clone()
            }
        };

        // Parse current config file & assert is same as prior parse
        match config.parse(&config_path.into_os_string().into_string().unwrap()) {
            Ok(cfg) => assert!(cfg.eq(&config)),
            Err(err) => panic!("Could not parse config: {}", err),
        }
    }
}
