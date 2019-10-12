// SPDX-License-Identifier: MIT

extern crate chrono;
extern crate clap;
extern crate dirs;
extern crate fern;
extern crate regex;
extern crate serde;
extern crate toml;

use clap::{App, Arg, ArgMatches};
use regex::Regex;
use std::fs::File;
use std::io::ErrorKind;

const DEFAULT_CONFIG_FILE: &str = "~/.config/ignore-ng/config";
#[derive(Deserialize, Debug)]
pub struct Config {
    pub config_file: String,
    pub gitignore_repo: String,
    pub repo_parent_dir: String,
    pub repo_path: String,
    pub repo_url: String,
}


// TODO: populate this
pub fn parse_config_file() {
    let config_file = File::open(DEFAULT_CONFIG_FILE).unwrap_or_else(|err| {
        if err.kind() == ErrorKind::NotFound {
            File::create(DEFAULT_CONFIG_FILE).expect("Could not create default config file")
        } else {
            // Panic?
            warn!("Could not open config file: {:?}", err);
        }
    });

    match config_file.read_to_string(&mut config_string) {
        Ok(size) => {
            if size > 0 {
                app_config = toml::from_str(&config_string.trim()).unwrap();
            }
            
            // TODO: review this
            info!("Config file is empty");
            let r_path = r_parent_dir.push(Regex::new(URL_PREFIX_REGEX)
                    .unwrap()
                    .replace(&app_config.repo_url, ""));

            r_parent_dir = dirs::cache_dir().unwrap();
            r_parent_dir.push("ignore-ng/repos");

            app_config = &mut Config {
                config_file: app_config.config_file,
                repo_url: default_gitignore_repo.to_string(),
                repo_parent_dir: r_parent_dir.into_os_string().to_str().unwrap(),
                repo_path: r_path.into_os_string().to_str().unwrap(),
            };

            // Populate with defaults & use defaults
        }

        Err(err) => panic!("Could not read config file contents: {:?}", err),
    }
}

pub fn parse_flags() -> Result<(ArgMatches<'static>, Config), ()> {
    let default_config_file = dirs::config_dir().unwrap();
    default_config_file.push("ignore-ng/config.toml");

    let config_file: &str;
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

    setup_logger(&matches)?;
    debug!("Logger is up");

    Ok(matches)
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
