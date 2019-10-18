// SPDX-License-Identifier: MIT

extern crate chrono;
extern crate dirs;
extern crate fern;
extern crate regex;
extern crate serde;
extern crate toml;

use clap::{App, Arg, ArgMatches};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::ErrorKind;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub core: CoreConfig,
    pub repo: RepoConfig,
}

// REF: https://mathiasbynens.be/demo/url-regex
// TODO: validate regex
const URL_PREFIX_REGEX: &str =
    r"#(?i)\b((?:[a-z][\w-]+:(?:/{1,3}|[a-z0-9%])|www\d{0,3}[.]|[a-z0-9.\-]+[.][a-z]{2,4}/)";
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CoreConfig {
    pub last_run: SystemTime,
}

pub fn parse_config_file(app_config: &mut Config, config_file_path: &str) {
    let default_gitignore_repo: &str = "https://github.com/github/gitignore";
    let r_path: &str;

    let config_string: String;

    let default_config_file: PathBuf;
    let r_parent_dir: PathBuf;

    let toml_config: Config;

    let config_file: File;

    default_config_file = dirs::config_dir().unwrap();
    default_config_file.push("ignore-ng/config.toml");

    /* if let Some(path) = dirs::config_dir() {
     *     path.push("ignore-ng/config.toml");
     *     default_config_file = path;
     * } */
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RepoConfig {
    pub repo_parent_dir: String,
    pub repo_path: String,
    pub repo_url: String,
}

    config_file = File::open(config_file_path).unwrap_or_else(|err| {
        if err.kind() == ErrorKind::NotFound {
            if config_file_path.eq(default_config_file.into_os_string().to_str().unwrap()) {
                File::create(config_file_path).expect("Could not create default config file")
            } else {
                panic!("Could not find config file: {:?}", err);
            }
        } else {
            panic!("Could not open config file: {:?}", err);
        }
    });

    match config_file.read_to_string(&mut config_string) {
        Ok(size) => {
            if size > 0 {
                // Returns here
                toml_config: Config = toml::from_str(&config_string.trim()).unwrap();
                app_config = &mut toml_config
            }

            info!("Config file is empty");
            r_path = &Regex::new(URL_PREFIX_REGEX)
                .unwrap()
                .replace(app_config.repo_url, "");

            r_parent_dir = dirs::cache_dir().unwrap();
            r_parent_dir.push("ignore-ng/repos");

            // Populate config with defaults
            app_config = &mut Config {
                repo_url: default_gitignore_repo,
                repo_parent_dir: r_parent_dir.into_os_string().to_str().unwrap(),
                repo_path: r_path,
            };
            debug!("Using default config values");

            // Write default config to file
            config_file
                .write_all(toml::to_string(&app_config).unwrap().as_bytes())
                .unwrap();
            debug!("Updated config file with config values")
        }
        Err(err) => panic!("Could not read config file contents: {:?}", err),
    }
}

pub fn parse_flags() -> Result<(ArgMatches<'static>, Config), ()> {
    let config_file: &str;

    let default_config_file: PathBuf;

    let mut app_config: Config;

    // env!("CARGO_PKG_VERSION")
    let matches = App::new("ignore-ng")
        .version(crate_version!())
        .about("Generated .gitignore files")
        .author("fisherprime")
        .arg(
            Arg::with_name("list")
                .short("l")
                .long("list")
                .help("List all available languages, tools & projects"),
        )
        .arg(
            Arg::with_name("template")
                .short("t")
                .long("templates")
                .help(
                "List language(s), tool(s) and/or project template(s) to generate .gitignore from")
                .takes_value(true),
        )
        .arg(Arg::with_name("config")
            .short("c").long("config").help("Specify alternative config file to use"))
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("Set the level of verbosity for logs: -v, -vv, -vvv"),
        )
        .get_matches();
    debug!("Parsed command flags");

    setup_logger(&matches).unwrap();
    debug!("Logger is up");

    default_config_file = dirs::config_dir().unwrap();
    default_config_file.push("ignore-ng/config.toml");

    if let Some(path) = matches.value_of("config") {
        parse_config_file(&mut app_config, path);
        debug!("Using user supplied config file path");
    } else {
        parse_config_file(
            &mut app_config,
            default_config_file.into_os_string().to_str().unwrap(),
        );
        debug!("Using default config file path");
    }

    Ok((matches, app_config))
}

fn setup_logger(matches: &ArgMatches) -> Result<(), fern::InitError> {
    let log_max_level = match matches.occurrences_of("verbosity") {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        2 => log::LevelFilter::Trace,
        _ => log::LevelFilter::Off,
    };

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log_max_level)
        .chain(std::io::stdout())
        // .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

/* #[cfg(test)]
 * mod tests {
 *     use super::*;
 *
 *     #[test]
 *     fn setup_logger_test() {
 *         assert!(asdasda)
 *     }
 * } */
