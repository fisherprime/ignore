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

/// Constant specifying the amount of seconds in a day as [`u64`].
const SECONDS_IN_DAY: u64 = 60 * 60 * 24;

/// Constant specifying the time to consider a repository's contents as state as [`u64`] (unsigned
/// 64-bit integer).
/// Set to 7 days.
const REPO_UPDATE_LIMIT: u64 = SECONDS_IN_DAY * 7;

/// Constant specifying the default gitignore template repo to use.
const GITIGNORE_DEFAULT_REPO: &str = "https://github.com/github/gitignore";

/// Constant specifying the cache subdirectory within the system's cache directory to store
/// gitignore template repositories.
const GITIGNORE_REPO_CACHE_SUBDIR: &str = "ignore/repos";

/// Constant specifying the location of the last run state file from some parent directory (i.e.
/// system cache directory).
const STATE_FILE_SPATH: &str = "ignore/.state";

/// Struct containing identifiers on the state of the binary's last run.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct State {
    /// Timestamp of the last time the binary was run.
    pub last_run: SystemTime,
}

/// Struct containing the runtime options parsed from a config file.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Config {
    /* /// Binary specific configuration options.
     * pub core: CoreConfig, */
    /// Repository specific configuration options.
    pub repo: RepoConfig,
}

/* /// Struct containing the config file's core (not repository related) runtime options.
 * #[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
 * pub struct CoreConfig {
 * } */

/// Struct containing the config file's common & array of repository specific runtime options.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct RepoConfig {
    /// Directory containing gitignore repositories.
    pub repo_parent_dir: String,

    /// [`RepoDetails`] for multiple template repositories.
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

    /// Previous runtime state as read from file.
    pub state: State,

    /// Exclusive operation specified by user.
    pub operation: Operation,

    /// Option used to auto-update cached gitignore tempalte repositories.
    pub needs_update: bool,

    /// Path to configuration file.
    pub config_path: String,

    /// Path to last runtime state file.
    pub state_path: String,

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
    /// Option for unknown operations.
    Else,
}

/// Enum containing runtime related filetypes.
pub enum RuntimeFile {
    /// Option to update the config file.
    ConfigFile,
    /// Option to update the state file
    StateFile,
}

impl State {
    /// Generates the default [`State`].
    pub fn new(now: SystemTime) -> State {
        State { last_run: now }
    }

    /// Parses state file contents & generates a [`State`] item.
    pub fn parse(self) -> Result<State, Box<dyn Error>> {
        let mut state_file_path = dirs::cache_dir().unwrap();
        state_file_path.push(STATE_FILE_SPATH);

        let read_bytes: usize;

        let mut state_string = String::new();

        let mut state_file: File;

        if !&state_file_path.exists() {
            create_file(&state_file_path)?;
        }

        state_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(state_file_path)?;
        read_bytes = state_file
            .read_to_string(&mut state_string)
            .unwrap_or_else(|_| 0);

        if read_bytes > 0 {
            if let Ok(state) = toml::from_str(state_string.trim()) {
                debug!("Done parsing state file");
                return Ok(state);
            }
        }

        info!("State file is empty");

        Ok(self.clone())
    }

    /// Updates the contents of the state file with the current [`State`].
    fn update_file(&self, state_file: &mut File) -> Result<(), Box<dyn Error>> {
        state_file.write_all(toml::to_string(&self)?.as_bytes())?;
        debug!("Updated state file");

        Ok(())
    }
}

impl Config {
    /// Generates the default [`Config`].
    pub fn new() -> Config {
        let default_gitignore_repo: String = GITIGNORE_DEFAULT_REPO.to_string();
        let r_path: String;

        let mut r_parent_dir: PathBuf;

        let gitignore_repo_path = Path::new(&default_gitignore_repo);
        let mut gitignore_repo_components: Vec<_> = gitignore_repo_path
            .components()
            .map(|comp| comp.as_os_str())
            .collect();

        if gitignore_repo_components.len().lt(&2) {
            r_path = format!(
                "undefined/{}",
                gitignore_repo_components.pop().unwrap().to_str().unwrap()
            );
        } else {
            r_path = format!(
                "{1}/{0}",
                gitignore_repo_components.pop().unwrap().to_str().unwrap(),
                gitignore_repo_components.pop().unwrap().to_str().unwrap()
            );
        }

        r_parent_dir = dirs::cache_dir().expect("Error obtaining system's cache directory");
        r_parent_dir.push(GITIGNORE_REPO_CACHE_SUBDIR);

        Config {
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

    /// Parses config file contents & generates a [`Config`] item.
    // Passing a reference to Config struct avoid taking ownership.
    fn parse(&self, config_file_path: &str) -> Result<Config, Box<dyn Error>> {
        debug!("Parsing config file");

        let read_bytes: usize;

        let mut config_string = String::new();

        let mut default_config_file: PathBuf;

        let mut config_file: File;

        default_config_file =
            dirs::config_dir().expect("Error obtaining system's config directory");
        default_config_file.push("ignore/config.toml");

        if !Path::new(config_file_path).exists()
            && Path::new(config_file_path).eq(&default_config_file)
        {
            create_file(&Path::new(config_file_path))?;
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
        self.update_file(&mut config_file)?;

        Ok(self.clone())
    }

    /// Updates the contents of the config file with the current [`Config`].
    fn update_file(&self, config_file: &mut File) -> Result<(), Box<dyn Error>> {
        config_file.write_all(toml::to_string(&self)?.as_bytes())?;
        debug!("Updated config file");

        Ok(())
    }
}

impl Options {
    /// Parses command arguments.
    pub fn parse() -> Result<Options, Box<dyn Error>> {
        debug!("Parsing command arguments & config file");

        let now = SystemTime::now();

        let mut config_file_path = String::new();
        let state_file_path: String;

        let mut default_config_file: PathBuf;
        let mut state_file_pathbuf: PathBuf;

        let mut app_config = Config::new();
        let mut app_state = State::new();
        let app_options: Options;

        let matches: ArgMatches;

        // `env!("CARGO_PKG_VERSION")` replaced with `crate_version!`
        matches = App::new("ignore")
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
                .help("Specify output filename, defaults to: gitignore.")
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
        default_config_file.push("ignore/config.toml");

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

        state_file_pathbuf = dirs::cache_dir().expect("Error obtaining system's cache directory");
        state_file_pathbuf.push(STATE_FILE_SPATH);
        state_file_path = state_file_pathbuf.into_os_string().into_string().unwrap();

        /* // Create repo_path from repo_url
         * let re = Regex::new(URL_PREFIX_REGEX)
         *     .unwrap()
         *     .replace(default_gitignore_repo, ""); */

        app_config = app_config
            .parse(&config_file_path)
            .map(|cfg| cfg)
            .unwrap_or_else(|err| {
                error!("Config parse error, using the default: {}", err);
                app_config.clone()
            });

        app_state = app_state.parse()?;

        app_options = Options {
            config: app_config,
            state: app_state.clone(),
            operation: get_operation(&matches),
            needs_update: check_staleness(&app_state.last_run, &now)?,
            config_path: config_file_path,
            state_path: state_file_path,
            output_file: matches
                .value_of("output")
                .unwrap_or("gitignore")
                .to_string(),
            templates: match matches.values_of("template") {
                Some(templates_arg) => templates_arg
                    .map(|tmpl| tmpl.to_string())
                    .collect::<Vec<String>>(),
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

    /// A wrapper function to allow saving a [`Config`]|[`State`] contained within an [`Options`] item.
    pub fn save_file(&self, file_type: RuntimeFile) -> Result<(), Box<dyn Error>> {
        let mut runtime_file: File;
        let file_path: String;

        file_path = match file_type {
            RuntimeFile::StateFile => self.state_path.clone(),
            RuntimeFile::ConfigFile => self.config_path.clone(),
        };

        debug!("Updating file: {}", file_path);

        runtime_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(file_path)?;
        runtime_file.set_len(0)?;

        match file_type {
            RuntimeFile::StateFile => self.state.update_file(&mut runtime_file)?,
            RuntimeFile::ConfigFile => self.config.update_file(&mut runtime_file)?,
        };

        Ok(())
    }
}

/// Creates a file defined by a filepath.
///
/// This function builds a filepath's directory heirarchy (if necessary) then creates the file
/// specified by the path.
fn create_file(file_path: &Path) -> Result<(), Box<dyn Error>> {
    info!("Creating file: {}", file_path.display());

    let file_dir = Path::new(&file_path).parent().unwrap();
    if !file_dir.is_dir() {
        DirBuilder::new().recursive(true).create(file_dir)?
    }

    File::create(file_path)?;

    Ok(())
}

/// Determines the operation specified in the user supplied arguments.
///
/// This function checks for the presence of user arguments as provided in the [`ArgMatches`]
/// struct created by [`clap`].
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
/// This function compares the current [`SystemTime`] to the last repository update time.
/// This function returns true (staleness state) should the time difference between the last
/// repo update & current run be greater than [`REPO_UPDATE_LIMIT`] or this be the first execution
/// of the binary.
/// Otherwise, this function returns false.
fn check_staleness(last_update: &SystemTime, now: &SystemTime) -> Result<bool, Box<dyn Error>> {
    let repos_are_stale = {
        ((now.duration_since(*last_update)? > Duration::new(REPO_UPDATE_LIMIT, 0))
            || (now.eq(last_update)))
    };

    Ok(repos_are_stale)
}

// REF: https://mathiasbynens.be/demo/url-regex
// TODO: validate regex
/* const URL_PREFIX_REGEX: &str = */
/* r"#(?i)\b((?:[a-z][\w-]+:(?:/{1,3}|[a-z0-9%])|www\d{0,3}[.]|[a-z0-9.\-]+[.][a-z]{2,4}/))"; */

/// Configures the [`fern`] logger.
///
/// This function configures the logger to output log messages using the ISO date format with
/// verbosity levels specified by the user arguments (within [`ArgMatches`]).
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

    match verbose {
        true => {
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
        }
        false => {
            fern::Dispatch::new()
                .format(|out, message, record| {
                    out.finish(format_args!("[{}] {}", record.level(), message))
                })
                .level(log_max_level)
                .chain(std::io::stdout())
                // .chain(fern::log_file("output.log")?)
                .apply()?;
        }
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

        let mut parent_dir = dirs::cache_dir().unwrap();
        parent_dir.push("ignore/repos");

        let hardcode_config = Config {
            repo: RepoConfig {
                repo_parent_dir: parent_dir.into_os_string().into_string().unwrap(),
                repo_dets: vec![RepoDetails {
                    auto_update: false,
                    ignore: false,
                    repo_url: GITIGNORE_DEFAULT_REPO.to_string(),
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
        config_path.push("ignore/config.toml");

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
