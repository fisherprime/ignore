// SPDX-License-Identifier: MIT

//! The config module defines elements necessary for the setup and configuration of the runtime
//! environment.

extern crate chrono;
extern crate dirs;
extern crate fern;
extern crate serde;
extern crate toml;

// use std::collections::btree_map::BTreeMap;
// use std::collections::hash_map::HashMap;
// use std::ffi::OsString;
// use std::io::ErrorKind;
use clap::{App, AppSettings, Arg, ArgMatches};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{DirBuilder, File, OpenOptions};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Constant specifying the time to consider a repository's contents as state as an unsigned 64 bit
/// integer.
/// Set to 7 days.
const REPO_UPDATE_LIMIT: u64 = 60 * 60 * 24 * 7;

/// Struct containing runtime options parsed from a config file.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Config {
    /// Binary specific configuration options.
    pub core: CoreConfig,

    /// Repository specific configuration options.
    pub repo: RepoConfig,
}

/// Struct containing the config file's core(not repo config) runtime options.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct CoreConfig {
    /// Timestamp of the last time the binary was run.
    pub last_run: SystemTime,
}

/// Struct containing the config file's common & array of repository specific runtime options.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct RepoConfig {
    /// Directory containing gitignore repositories.
    pub repo_parent_dir: String,

    /// Details for multiple/single template repository.
    pub repo_dets: Vec<RepoDetails>,
}

/// Struct containing the config file's repository specific runtime options.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct RepoDetails {
    /// Choice for automatically updating the cached repo.
    pub auto_update: bool,

    /// Choice for ignoring repository usage for any task.
    pub ignore: bool,

    /// Relative path (to repo_parent_dir) of gitignore template repo to use.
    pub repo_path: String,

    /// URL of git repository containing gitignore templates.
    pub repo_url: String,
}

/// Struct containing runtime options gathered from the config file and command arguments.
#[derive(Debug, Clone)]
pub struct Options {
    /// Config read from file.
    pub config: Config,

    /// Exclusive operation specified by user.
    pub operation: Operation,

    /// Option used to auto-update cached gitignore tempalte repositories.
    pub needs_update: bool,

    /// Path to configuration file.
    pub config_path: String,

    /// Path to output generated gitignore.
    pub output_file: String,

    /// List of templates user desires to use in gitignore generation.
    pub templates: Vec<String>,
}

/// Enum containing exclusive operations that can be performed.
#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    /// Option to list available templates.
    ListTemplates,
    /// Option to update repository.
    UpdateRepo,
    /// Option to generate gitignore file.
    GenerateGitignore,
    /* /// Option to skip running any operation.
     * Skip, */
    /// Option for unknown operation.
    Else,
}

impl Config {
    /// Function used to generate the default Config struct.
    pub fn new() -> Config {
        let default_gitignore_repo: String = "https://github.com/github/gitignore".to_string();
        let r_path: String;

        let mut r_parent_dir: PathBuf;

        let now = SystemTime::now();

        // TODO: fix, messy_repo_path.
        // Get repo_path as defined in the Options struct.
        let gitignore_repo_split: Vec<&str> = default_gitignore_repo.split('/').collect();
        let gitignore_split_len = gitignore_repo_split.len();

        r_path = format!(
            "{}/{}",
            gitignore_repo_split[gitignore_split_len - 2],
            gitignore_repo_split[gitignore_split_len - 1]
        );
        // TODO: end of messy_repo_path.
        r_parent_dir = dirs::cache_dir().expect("Error obtaining system's cache directory");
        r_parent_dir.push("ignore-ng/repos");

        Config {
            core: CoreConfig {
                // Sort out duration since error???
                last_run: now - Duration::new(0, 500),
            },
            repo: RepoConfig {
                repo_parent_dir: r_parent_dir.into_os_string().into_string().unwrap(),
                repo_dets: vec![RepoDetails {
                    auto_update: false,
                    ignore: false,
                    repo_url: default_gitignore_repo,
                    repo_path: r_path,
                }],
            },
        }
    }

    // Function used to parse config file contents, populating a Config struct.
    // Passing a reference to Config struct avoid taking ownership.
    fn parse(&self, config_file_path: &str) -> Result<Config, Box<dyn Error>> {
        debug!("Parsing config file");

        let read_bytes: usize;

        let mut config_string = String::new();

        let mut default_config_file: PathBuf;

        let mut config_file: File;

        default_config_file =
            dirs::config_dir().expect("Error obtaining system's config directory");
        default_config_file.push("ignore-ng/config.toml");

        /* match OpenOptions::new()
         *     .read(true)
         *     .write(true)
         *     .create(true)
         *     .open(config_file_path)
         * {
         *     Ok(conf) => config_file = conf,
         *     Err(err) => {
         *         if err.kind() == ErrorKind::NotFound {
         *             self.create_default_config_file(
         *                 &default_config_file,
         *                 &Path::new(config_file_path),
         *             )?;
         *         } else {
         *             // panic!("Could not find config file: {:?}", err);
         *             return Err(Box::new(err));
         *         }
         *     }
         * }; */

        if !Path::new(config_file_path).exists() {
            self.create_default_config_file(&default_config_file, &Path::new(config_file_path))?;
        }

        config_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(config_file_path)?;
        read_bytes = config_file
            .read_to_string(&mut config_string)
            .unwrap_or_else(|_| 0);

        if read_bytes > 0 {
            match toml::from_str(config_string.trim()) {
                Ok(cfg) => {
                    debug!("Done parsing config file");
                    return Ok(cfg);
                }
                Err(_) => {
                    info!("Backing up config file");
                    std::fs::copy(config_file_path, format!("{}.bak", config_file_path))?;
                }
            }
        }

        info!("Config file is empty, using default config values");
        self.update_config_file(&mut config_file)?;

        Ok(self.clone())
    }

    fn update_config_file(&self, config_file: &mut File) -> Result<(), Box<dyn Error>> {
        config_file.write_all(toml::to_string(&self)?.as_bytes())?;
        debug!("Updated config file");

        Ok(())
    }

    fn create_default_config_file(
        &self,
        default_config_file: &Path,
        config_file_path: &Path,
    ) -> Result<(), Box<dyn Error>> {
        if config_file_path.eq(default_config_file) {
            info!("Creating default config file");

            let conf_dir = Path::new(&config_file_path).parent().unwrap();
            if !conf_dir.is_dir() {
                DirBuilder::new().recursive(true).create(conf_dir)?
            }

            File::create(config_file_path)?;
        }

        Ok(())
    }
}

impl Options {
    // Parse command arguments.
    pub fn parse() -> Result<Options, Box<dyn Error>> {
        debug!("Parsing command arguments & config file");

        let mut config_file_path = String::new();

        let mut default_config_file: PathBuf;

        let app_config = Config::new();
        let app_options: Options;

        let matches: ArgMatches;

        // `env!("CARGO_PKG_VERSION")` replaced with `crate_version!`
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
            config: app_config
                .parse(&config_file_path)
                .map(|cfg| cfg)
                .unwrap_or_else(|err| {
                    error!("Config parse error, using the default: {}", err);
                    app_config.clone()
                }),
            operation: get_operation(&matches),
            needs_update: check_staleness(&app_config.core.last_run)?,
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

    pub fn save_config(self) -> Result<(), Box<dyn Error>> {
        let mut config_file: File;

        debug!("Updating config in file path: {}", self.config_path);

        config_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(self.config_path)?;

        config_file.set_len(0)?;

        self.config.update_config_file(&mut config_file)?;

        Ok(())
    }
}

/// Determines the operation specified by the user.
///
/// This function checks for the presence of user arguments as provided in the ArgMatches
/// struct created by clap.
fn get_operation(matches: &ArgMatches) -> Operation {
    if matches.is_present("template") {
        return Operation::GenerateGitignore;
    }

    if matches.is_present("list") {
        return Operation::ListTemplates;
    }

    if matches.is_present("update") {
        return Operation::UpdateRepo;
    }

    Operation::Else
}

/// Checks for staleness of the cached gitignore template repositories.
///
/// This function compares the current SystemTime to the last repository update time.
/// This function returns true (staleness state) should the difference be greater than
/// REPO_UPDATE_LIMIT; otherwise, false.
fn check_staleness(last_update: &SystemTime) -> Result<bool, Box<dyn Error>> {
    let now = SystemTime::now();
    let update_test = {
        ((now.duration_since(*last_update)? > Duration::new(REPO_UPDATE_LIMIT, 0))
            || (now.duration_since(*last_update)? == Duration::new(0, 500)))
    };

    if update_test {
        return Ok(true);
    }

    Ok(false)
}

// REF: https://mathiasbynens.be/demo/url-regex
// TODO: validate regex
/* const URL_PREFIX_REGEX: &str = */
/* r"#(?i)\b((?:[a-z][\w-]+:(?:/{1,3}|[a-z0-9%])|www\d{0,3}[.]|[a-z0-9.\-]+[.][a-z]{2,4}/))"; */

/// Configures the fern logger.
///
/// This function configures the logger to output log messages using the ISO date format with
/// verbosity levels specified by the user arguments (within ArgMatches).
/// The arguments set the output verbosity for this crate to a maximum log level of either: Info,
/// Debug, Trace level entries of none altogether.
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

    #[test]
    /// Assert correctness of the default config options
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
                    auto_update: false,
                    ignore: false,
                    repo_url: "https://github.com/github/gitignore".to_string(),
                    repo_path: "github/gitignore".to_string(),
                }],
            },
        };

        assert!(hardcode_config.repo.eq(&config.repo));
    }

    #[test]
    /// Assert correctness of parsed default config file.
    fn config_file_parse_test() {
        let mut config = Config::new();

        let mut config_path = dirs::config_dir().unwrap();
        config_path.push("ignore-ng/config.toml");

        // Parse default config file, populating it with the default config if non-existent.
        config = config
            .parse(&config_path.clone().into_os_string().into_string().unwrap())
            .map(|cfg| cfg)
            .unwrap_or_else(|err| {
                error!("Config parse error, using the default: {}", err);
                config.clone()
            });

        // Parse current config file & assert is similar to the default.
        config
            .parse(&config_path.into_os_string().into_string().unwrap())
            .map(|cfg| assert!(cfg.eq(&config)))
            .unwrap_or_else(|err| panic!("Could not parse config: {}", err));
    }
}
