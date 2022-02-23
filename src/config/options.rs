// SPDX-License-Identifier: MIT

//! The `options` module defines elements necessary for the configuration of [`Options`] (contains
//! the runtime options).

use super::{config::Config, state::State};

use std::error::Error as StdErr;
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
    /// Load options from the arguments, config file & state file.
    pub fn load() -> Result<Options, Box<dyn StdErr>> {
        use super::cli::setup_clap;
        use super::logger::setup_logger;

        debug!("Parsing command arguments & config file");

        let now = SystemTime::now();

        let mut app_config = Config::default();
        let app_state = State::new(&now).load()?;

        let matches = setup_clap()?;
        setup_logger(&matches)?;

        app_config = app_config
            .load(&matches.value_of("config").unwrap_or_default().to_owned())
            .unwrap_or_else(|err| {
                error!("Config load error, using the default: {}", err);
                app_config
            });

        let is_stale = app_state.check_staleness(&now)?;
        let mut app_options = Options {
            config: app_config,
            state: app_state,
            needs_update: is_stale,
            operation: Operation::Else,
            gitignore_output_file: "".to_owned(),
            templates: ["".to_string()].to_vec(),
        };
        app_options.configure_operation(&matches);

        debug!(
            "Loaded command arguments & config file, options: {:#?}",
            app_options
        );

        Ok(app_options)
    }

    /// Configures the `Options` to execute subcommand selected by the user.
    ///
    /// This function checks for the presence of [`clap::Subcommand`]s & [`clap::Arg`]s as provided
    /// in the [`clap::ArgMatches`] struct.
    fn configure_operation(&mut self, matches: &ArgMatches) {
        match matches.subcommand() {
            Some(("list", _)) => self.operation = Operation::ListAvailableTemplates,
            Some(("update", _)) => self.operation = Operation::UpdateRepositories,
            Some(("generate", sub_matches)) => {
                self.operation = Operation::GenerateGitignore;

                self.gitignore_output_file = sub_matches
                    .value_of("output")
                    .unwrap_or_default()
                    .to_owned();
                match sub_matches.values_of("template") {
                    Some(templates_arg) => {
                        self.templates = templates_arg
                            .map(|tmpl| tmpl.to_owned())
                            .collect::<Vec<String>>()
                    }
                    _ => {}
                }
            }
            _ => self.operation = Operation::Else,
        }
    }
}
