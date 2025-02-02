// SPDX-License-Identifier: MIT

//! The `cli` module defines functions necessary for the setup of [`clap`] and [`fern`].

use std::error::Error as StdErr;
use std::ffi::OsString;
use std::path::PathBuf;

use clap::{Arg, ArgAction, Command};
use clap_complete::Shell;

use crate::errors::Error;
use crate::errors::ErrorKind;

pub const DEFAULT_OUTPUT_FILE: &str = "gitignore";
const DEFAULT_CONFIG_PATH: &str = "ignore/config.toml";

pub const COMPLETIONS_SUBCMD: &str = "completions";
pub const LIST_SUBCMD: &str = "list";
pub const UPDATE_SUBCMD: &str = "update";
pub const GENERATE_SUBCMD: &str = "generate";

lazy_static! {
    static ref CFG_FILE_PATH_BUF: PathBuf = {
        let mut default_config_file_path = PathBuf::new();
        if let Some(v) = dirs_next::config_dir() {
            default_config_file_path = v
        }
        default_config_file_path.push(DEFAULT_CONFIG_PATH);
        default_config_file_path
    };
    static ref CFG_FILE: &'static str = CFG_FILE_PATH_BUF.to_str().unwrap_or(DEFAULT_CONFIG_PATH);
}

/// Obtains the default config file path for the executable's operating system.
#[allow(dead_code)]
pub fn get_config_file_path() -> Result<OsString, Box<dyn StdErr>> {
    let mut default_config_file_path: PathBuf;
    match dirs_next::config_dir() {
        Some(v) => default_config_file_path = v,
        None => return Err(Box::new(Error::from(ErrorKind::LocateConfigDir))),
    }
    default_config_file_path.push(DEFAULT_CONFIG_PATH);

    Ok(default_config_file_path.into_os_string())
}

/// Builds a [`clap::Command`].
// pub fn build_cli() -> Result<Command<'static>, Box<dyn StdErr>> {
pub fn build_cli() -> Command {
    Command::new(crate_name!())
        .arg_required_else_help(true)
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!())
        .arg(
            Arg::new("config")
            .help("Load configuration from FILE")
            .short('c')
            .long("config")
            .value_name("FILE")
            .default_value(*CFG_FILE)
            .value_parser( value_parser!(String))
        )
        .arg(
            Arg::new("verbosity")
            .help("Set the level of verbosity: -v or -vv")
            .short('v')
            .long("verbose")
            .action(ArgAction::Count)
            // .multiple_occurrences(true)
        ).subcommand(
        Command::new(COMPLETIONS_SUBCMD)
        .arg_required_else_help(true)
        .about("Generate tab completion scripts")
        .arg(
            Arg::new("shell")
            .help("Specify shell to generate completion script for")
            .value_name("SHELL")
            .value_parser(value_parser!(Shell))
        )
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
                .value_parser(value_parser!(PathBuf))
            )
            .arg(
                Arg::new("template")
                .help("Case sensitive (space-separated) list of TEMPLATE(s) to use in generating the gitignore file")
                .short('t')
                .long("templates")
                .num_args(1..)
                .value_name("TEMPLATE")
                .action(ArgAction::Append)
            )
        )
}
