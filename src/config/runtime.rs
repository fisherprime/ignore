// SPDX-License-Identifier: MIT

//! The `options` module defines elements necessary for the configuration of [`RuntimeConfig`] (contains
//! the runtime options).

use crate::config::cli::{build_cli, APP_NAME};

use super::{configs::Config, state::State};

use std::error::Error as StdErr;
use std::str::FromStr;
use std::time::SystemTime;

use clap::ArgMatches;
use clap_complete::Shell;

/// `struct` containing runtime options gathered from the config file and command arguments.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
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

    /// Shell to generate completions for.
    pub completion_shell: Shell,

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

/// Default implementation for [`RuntimeConfig`].
impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            matches: ArgMatches::default(),
            config: Config::default(),
            state: State::default(),
            operation: Operation::Else,
            gitignore_output_file: "".to_owned(),

            completion_shell: Shell::Zsh,

            templates: vec!["".to_string()],
        }
    }
}

/// Method implementations for [`RuntimeConfig`].
impl RuntimeConfig {
    /// Load options from the arguments, config file & state file.
    pub fn load(&mut self) -> Result<RuntimeConfig, Box<dyn StdErr>> {
        use super::logger::setup_logger;

        debug!("parsing command arguments & config file");

        let now = SystemTime::now();
        self.state = State::new(&now).load()?;

        self.matches = build_cli().get_matches();
        debug!("parsed command flags");
        setup_logger(&self.matches)?;

        self.config
            .load(
                &self
                    .matches
                    .get_one::<String>("config")
                    .expect("failed to use default config")
                    .to_owned(),
            )
            .unwrap_or_else(|err| {
                error!("config load error, using the default: {}", err);
                Config::default()
            });
        self.configure_operation();

        debug!(
            "loaded command arguments & config file, options: {:#?}",
            self
        );

        Ok(self.clone())
    }

    /// Configures the `RuntimeConfig` to execute subcommand selected by the user.
    ///
    /// This function checks for the presence of [`clap::Subcommand`]s & [`clap::Arg`]s as provided
    /// in the [`clap::ArgMatches`] struct.
    fn configure_operation(&mut self) {
        use crate::config::cli::{COMPLETIONS_SUBCMD, GENERATE_SUBCMD, LIST_SUBCMD, UPDATE_SUBCMD};
        match self.matches.subcommand() {
            Some((LIST_SUBCMD, _)) => self.operation = Operation::ListAvailableTemplates,
            Some((UPDATE_SUBCMD, _)) => self.operation = Operation::UpdateRepositories,
            Some((GENERATE_SUBCMD, sub_matches)) => {
                self.operation = Operation::GenerateGitignore;

                self.gitignore_output_file = sub_matches
                    .get_one::<String>("output")
                    .expect("failed to use default output")
                    .to_owned();
                if let Some(templates_arg) = sub_matches.get_many::<String>("template") {
                    self.templates = templates_arg
                        .map(|tmpl| tmpl.to_owned())
                        .collect::<Vec<String>>()
                }
            }
            Some((COMPLETIONS_SUBCMD, sub_matches)) => {
                self.operation = Operation::GenerateCompletions;
                self.completion_shell = Shell::from_str(
                    sub_matches
                        .get_one::<String>("shell")
                        .expect("failed to use default shell"),
                )
                .unwrap_or(Shell::Zsh);
            }
            _ => self.operation = Operation::Else,
        }
    }

    /// Generates completions for shells defined in [`clap_complete::Shell`].
    pub fn generate_completions(&mut self) -> Result<(), Box<dyn StdErr>> {
        use clap_complete::generate;
        use std::io;

        generate(
            self.completion_shell,
            &mut build_cli(),
            APP_NAME,
            &mut io::stdout(),
        );

        Ok(())
    }
}
