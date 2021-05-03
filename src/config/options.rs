// SPDX-License-Identifier: MIT

//! The `options` module defines elements necessary for the configuration of [`Options`] (contains
//! the runtime options).

use super::{config_file::Config, state::State};

use std::error::Error as StdErr;
use std::path::Path;
use std::time::SystemTime;

use clap::ArgMatches;

/// `struct` containing runtime options gathered from the config file and command arguments.
#[derive(Debug, Clone)]
pub struct Options {
    /// Config read from file.
    pub config: Config,

    /// Previous runtime state as read from file.
    pub state: State,

    /// Exclusive operation specified by user.
    pub operation: Operation,

    /// Option used to auto-update cached gitignore template repositories.
    pub needs_update: bool,

    /// Path to output generated gitignore.
    pub gitignore_output_file: String,

    /// List of templates user desires to use in gitignore generation.
    pub templates: Vec<String>,
}

/// `enum` containing exclusive operations that can be performed.
#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    /// Option to list available templates.
    ListAvailableTemplates,
    /// Option to update repository.
    UpdateRepositories,
    /// Option to generate gitignore file.
    GenerateGitignore,
    /// Option for unknown operations.
    Else,
}

/// Method implementations for [`Options`].
impl Options {
    /// Parses command arguments.
    pub fn parse() -> Result<Options, Box<dyn StdErr>> {
        use super::setup::{setup_clap, setup_logger};

        debug!("Parsing command arguments & config file");

        let now = SystemTime::now();

        let mut config_file_path = String::new();

        let mut app_config = Config::default();
        let app_state = State::new(&now).parse()?;

        let mut matches = ArgMatches::new();

        setup_clap(&mut matches);

        setup_logger(&matches)?;

        let mut default_config_file_path =
            dirs_next::config_dir().expect("Error obtaining system's config directory");
        default_config_file_path.push("ignore/config.toml");

        if let Some(path) = matches.value_of("config") {
            if Path::new(path).exists() {
                config_file_path = path.to_owned();
                debug!("Using user supplied config file path");
            } else if let Some(cfg_path) = default_config_file_path.into_os_string().to_str() {
                config_file_path = cfg_path.to_owned();
                debug!("Using default config file path");
            }
        } else {
            if let Some(cfg_path) = default_config_file_path.into_os_string().to_str() {
                config_file_path = cfg_path.to_owned();
            }

            debug!("Using default config file path");
        }

        app_config = app_config.parse(&config_file_path).unwrap_or_else(|err| {
            error!("Config parse error, using the default: {}", err);
            app_config
        });

        let repo_staleness = app_state.check_staleness(&now)?;
        let app_options = Options {
            config: app_config,
            state: app_state,
            operation: get_operation(&matches),
            needs_update: repo_staleness,
            gitignore_output_file: matches.value_of("output").unwrap_or("gitignore").to_owned(),
            templates: match matches.values_of("template") {
                Some(templates_arg) => templates_arg
                    .map(|tmpl| tmpl.to_owned())
                    .collect::<Vec<String>>(),
                None => ["".to_string()].to_vec(),
            },
        };
        debug!(
            "Parsed command arguments & config file, options: {:#?}",
            app_options
        );

        Ok(app_options)
    }
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
        return Operation::ListAvailableTemplates;
    }

    if matches.is_present("update") {
        return Operation::UpdateRepositories;
    }

    Operation::Else
}
