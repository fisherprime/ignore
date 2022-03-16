// SPDX-License-Identifier: MIT

//! The `cli` module defines functions necessary for the setup of [`clap`] and [`fern`].

use std::error::Error as StdErr;
use std::path::PathBuf;

use clap::ArgMatches;

use crate::errors::Error;
use crate::errors::ErrorKind;

pub const APP_NAME : &str = "ignore";

const DEFAULT_OUTPUT_FILE: &str = "gitignore";
const DEFAULT_CONFIG_PATH: &str = "ignore/config.toml";

pub const COMPLETIONS_SUBCMD: &str = "completions";
pub const LIST_SUBCMD: &str = "list";
pub const UPDATE_SUBCMD: &str = "update";
pub const GENERATE_SUBCMD: &str = "generate";

/// Configures [`clap`].
///
/// This function configures [`clap`] then calls [`clap::App::get_matches`] on the result to yield
/// a [`clap::ArgMatches`] item.
pub fn setup_cli() -> Result<ArgMatches, Box<dyn StdErr>> {
    use clap::{Arg, Command};

    let mut default_config_file_path: PathBuf;
    match dirs_next::config_dir() {
        Some(v) => default_config_file_path = v,
        None => return Err(Box::new(Error::from(ErrorKind::LocateConfigDir))),
    }
    default_config_file_path.push(DEFAULT_CONFIG_PATH);

    let matches = Command::new(APP_NAME)
        .arg_required_else_help(true)
        .version(crate_version!())
        .about("A gitignore generator")
        .author("fisherprime")
        .arg(
            Arg::new("config")
            .help("Load configuration from FILE")
            .short('c')
            .long("config")
            .value_name("FILE")
            .default_value(default_config_file_path.into_os_string().to_str().unwrap_or(DEFAULT_CONFIG_PATH))
            .takes_value(true)
        )
        .arg(
            Arg::new("verbosity")
            .help("Set the level of verbosity: -v or -vv")
            .short('v')
            .long("verbose")
            .multiple_occurrences(true)
        ).subcommand(
        Command::new(COMPLETIONS_SUBCMD)
        .about("Generate tab completion scripts")
        )
        .subcommand(
            Command::new(UPDATE_SUBCMD)
            .about("Update the gitignore template repo(s)")              
        )
        .subcommand(
            Command::new(LIST_SUBCMD)
            .about("List available languages, tools & projects")
        )
        .subcommand(
            Command::new(GENERATE_SUBCMD)
            .arg_required_else_help(true)
            .about("Generate gitignore file")
            .arg(
                Arg::new("output")
                .help("Specify output FILE")
                .default_value(DEFAULT_OUTPUT_FILE)
                .short('o')
                .long("output")
                .value_name("FILE")
                .takes_value(true)
            )
            .arg(
                Arg::new("template")
                .help("Case sensitive (space-separated) list of TEMPLATE(s) to use in generating the gitignore file")
                .short('t')
                .long("templates")
                .value_name("TEMPLATE")
                .takes_value(true)
                .multiple_occurrences(true)
            )           
            )
            .get_matches();
    debug!("Parsed command flags");

    return Ok(matches);
}
