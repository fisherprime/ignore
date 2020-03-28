// SPDX-License-Identifier: MIT

//! The `config` module defines elements necessary for the setup and configuration of the runtime
//! environment.

use super::{config::Config, state::State};

use std::error::Error as StdErr;
use std::path::Path;
use std::time::{Duration, SystemTime};

use clap::ArgMatches;

/// Constant specifying the amount of seconds in a day as [`u64`].
const SECONDS_IN_DAY: u64 = 60 * 60 * 24;

/// Constant specifying the time to consider a repository's contents as state as [`u64`] (unsigned
/// 64-bit integer).
const REPO_UPDATE_LIMIT: u64 = SECONDS_IN_DAY * 7;

/// `struct` containing runtime options gathered from the config file and command arguments.
#[derive(Debug, Clone)]
pub struct Options {
    /// Config read from file.
    pub config: Config,

    /// Previous runtime state as read from file.
    pub state: State,

    /// Exclusive operation specified by user.
    pub operation: Operation,

    /// Option used to auto-update cached gitignore tempalate repositories.
    pub needs_update: bool,

    /// Path to output generated gitignore.
    pub output_file: String,

    /// List of templates user desires to use in gitignore generation.
    pub templates: Vec<String>,
}

/// `enum` containing exclusive operations that can be performed.
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

/// Method implementations for [`config::Options`].
impl Options {
    /// Parses command arguments.
    pub fn parse() -> Result<Options, Box<dyn StdErr>> {
        use super::setup::{setup_clap, setup_logger};

        debug!("Parsing command arguments & config file");

        let now = SystemTime::now();

        let mut config_file_path = String::new();

        let mut app_config = Config::default();
        let app_state = State::default().parse()?;

        let mut matches = ArgMatches::new();

        setup_clap(&mut matches);

        setup_logger(&matches)?;

        let mut default_config_path =
            dirs::config_dir().expect("Error obtaining system's config directory");
        default_config_path.push("ignore/config.toml");

        if let Some(path) = matches.value_of("config") {
            if Path::new(path).exists() {
                config_file_path = path.to_owned();
                debug!("Using user supplied config file path");
            } else {
                if let Some(cfg_path) = default_config_path.into_os_string().to_str() {
                    config_file_path = cfg_path.to_owned();
                    debug!("Using default config file path");
                }
            }
        } else {
            if let Some(cfg_path) = default_config_path.into_os_string().to_str() {
                config_file_path = cfg_path.to_owned();
            }

            debug!("Using default config file path");
        }

        app_config = app_config.parse(&config_file_path).unwrap_or_else(|err| {
            error!("Config parse error, using the default: {}", err);
            app_config
        });

        let app_options = Options {
            config: app_config,
            state: app_state.clone(),
            operation: get_operation(&matches),
            needs_update: check_staleness(&app_state.last_run, &now)?,
            output_file: matches.value_of("output").unwrap_or("gitignore").to_owned(),
            templates: match matches.values_of("template") {
                Some(templates_arg) => templates_arg
                    .map(|tmpl| tmpl.to_owned())
                    .collect::<Vec<String>>(),
                None => ["".to_string()].to_vec(),
            },
        };
        debug!(
            "Parsed command arguments & config file, options: {:?}",
            app_options
        );

        Ok(app_options)
    }
}

/// Checks for staleness of the cached gitignore template repositories.
///
/// This function compares the current [`SystemTime`] to the last repository update time.
/// This function returns `true` (staleness state) should the time difference between the last
/// repo update & current run be greater than [`REPO_UPDATE_LIMIT`] or this be the first execution
/// of the binary.
/// Otherwise, this function returns` false`.
pub fn check_staleness(
    last_update: &SystemTime,
    now: &SystemTime,
) -> Result<bool, Box<dyn StdErr>> {
    let repos_are_stale = {
        (now.duration_since(*last_update)? > Duration::new(REPO_UPDATE_LIMIT, 0))
            || now.eq(last_update)
    };

    Ok(repos_are_stale)
}

/// Determines the operation specified in the user supplied arguments.
///
/// This function checks for the presence of user arguments as provided in the [`clap::ArgMatches`]
/// struct.
pub fn get_operation(matches: &ArgMatches) -> Operation {
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
