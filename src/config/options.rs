// SPDX-License-Identifier: MIT

//! The `options` module defines elements necessary for the configuration of [`Options`] (contains
//! the runtime options).

use crate::config::cli::setup_cli;

use super::{config::Config, state::State};

use std::error::Error as StdErr;
use std::time::SystemTime;

use clap::ArgMatches;

/// `struct` containing runtime options gathered from the config file and command arguments.
#[derive(Debug, Clone)]
pub struct Options {
    /// Argument as read by [`clap`].
    matches: ArgMatches,

    /// Config read from file.
    pub config: Config,

    /// Previous runtime state as read from file.
    pub state: State,

    /// Exclusive operation specified by user.
    pub operation: Operation,

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
    /// Option to generate shell completion scripts.
    GenerateCompletions,
    /// Option for unknown operations.
    Else,
}

/// Default implementation for [`Options`].
impl Default for Options {
    fn default() -> Self {
        return Self {
            matches: ArgMatches::default(),
            config: Config::default(),
            state: State::default(),
            operation: Operation::Else,
            gitignore_output_file: "".to_owned(),

            templates: ["".to_string()].to_vec(),
        };
    }
}

/// Method implementations for [`Options`].
impl Options {
    /// Load options from the arguments, config file & state file.
    pub fn load(&mut self) -> Result<Options, Box<dyn StdErr>> {
        use super::logger::setup_logger;

        debug!("Parsing command arguments & config file");

        let now = SystemTime::now();

        self.state = State::new(&now).load()?;

        self.matches = setup_cli()?;
        setup_logger(&self.matches)?;

        self.config
            .load(
                &self
                    .matches
                    .value_of("config")
                    .unwrap_or_default()
                    .to_owned(),
            )
            .unwrap_or_else(|err| {
                error!("Config load error, using the default: {}", err);
                Config::default()
            });
        self.configure_operation();

        debug!(
            "Loaded command arguments & config file, options: {:#?}",
            self
        );

        Ok(self.clone())
    }

    /// Configures the `Options` to execute subcommand selected by the user.
    ///
    /// This function checks for the presence of [`clap::Subcommand`]s & [`clap::Arg`]s as provided
    /// in the [`clap::ArgMatches`] struct.
    fn configure_operation(&mut self) {
        match self.matches.subcommand() {
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
            Some(("generate_completions", _)) => {
                self.operation = Operation::GenerateCompletions
            }
            _ => self.operation = Operation::Else,
        }
    }

    pub fn generate_completions(&mut self) {
        /* use clap_complete::{generate, shells::Bash};
         * use std::io;
         * generate(Bash, &mut setup_cli().unwrap(), "ignore", &mut io::stdout()); */
        // TODO: Implement.
    }
}
