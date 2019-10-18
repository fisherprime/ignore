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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RepoConfig {
    pub repo_parent_dir: String,
    pub repo_path: String,
    pub repo_url: String,
}

pub struct Options<'a> {
    pub generate_gitignore: bool,
    pub list_templates: bool,
    pub update_repo: bool,

    pub config_path: String,

    pub templates: Vec<&'a str>,

    pub template_paths: BTreeMap<&'a str, &'a str>,
}

impl Config {
    pub fn parse<'main>() -> Option<(Config, Options<'main>)> {
        let mut app_config: Config;
        let app_options: Options;

        let matches: ArgMatches;

        let default_gitignore_repo: &str = "https://github.com/github/gitignore";
        let r_path: &str;

        let mut r_parent_dir: PathBuf;

        let now = SystemTime::now();

        // env!("CARGO_PKG_VERSION")
        matches = App::new("ignore-ng")
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
                .help("Set the level of verbosity for logs: -v, -vv"),
        )
        .get_matches();
        debug!("Parsed command flags");

        setup_logger(&matches).unwrap();
        debug!("Logger is up");

        /*
         *     let re = Regex::new(URL_PREFIX_REGEX)
         *         .unwrap()
         *         .replace(default_gitignore_repo, ""); */

        // TODO: fix, messy section
        let gitignore_repo_split: Vec<&str> = default_gitignore_repo.split('/').collect();
        let gitignore_split_len = gitignore_repo_split.len();
        let format_path = format!(
            "{}/{}",
            gitignore_repo_split[gitignore_split_len - 2],
            gitignore_repo_split[gitignore_split_len - 1]
        );

        r_path = format_path.trim();
        r_parent_dir = dirs::cache_dir().unwrap();
        r_parent_dir.push("ignore-ng/repos");

        app_config = Config {
            core: CoreConfig {
                // Sort out duration since error
                last_run: now - Duration::new(0, 500),
            },
            repo: RepoConfig {
                repo_url: String::from(default_gitignore_repo),
                repo_parent_dir: r_parent_dir.into_os_string().into_string().unwrap(),
                repo_path: String::from(r_path),
            },
        };

        app_options = Options {
            generate_gitignore: matches.is_present("template"),
            list_templates: matches.is_present("list"),
            update_repo: (now.duration_since(app_config.core.last_run).unwrap()
                > Duration::new(REPO_UPDATE_LIMIT, 0))
                || (now.duration_since(app_config.core.last_run).unwrap() == Duration::new(0, 500)),
            config_path: "".to_string(),
            templates: match matches.values_of("template") {
                /*                 Some(templates_vec) => {
                 *                     // TODO: fix, Error 515, cannot borrow local variable, function param or temporary variable
                 *                     let mut new = Vec::<&str>::new();
                 *                     for item in templates_vec.collect::<Vec<&str>>() {
                 *                         new.push(&item);
                 *                     }
                 *
                 *                     new
                 *
                 *                     // templates_vec.collect::<Vec<&str>>()
                 *                 } */
                Some(_) => [""].to_vec(),
                None => [""].to_vec(),
            },
            template_paths: BTreeMap::<&str, &str>::new(),
        };

        // Beware of move at this point
        let app_config_clone = app_config.clone();
        if let Some(cfg) = app_config_clone.parse_config_file(&matches) {
            debug!("{:?}", &cfg);
            app_config = cfg;
        }

        Some((app_config, app_options))
    }

    fn parse_config_file(self, matches: &ArgMatches) -> Option<Config> {
        let config: Config;

        let mut config_string = String::new();
        let mut config_file_path = String::new();

        let mut default_config_file: PathBuf;

        let def_cfg_os_str: OsString;

        let mut config_file: File;

        let read_bytes: usize;

        default_config_file = dirs::config_dir().unwrap();
        default_config_file.push("ignore-ng/config.toml");

        if let Some(path) = matches.value_of("config") {
            config_file_path = path.to_string();

            debug!("Using user supplied config file path");
        } else {
            def_cfg_os_str = default_config_file.clone().into_os_string();

            if let Some(cfg_str) = def_cfg_os_str.to_str() {
                config_file_path = cfg_str.to_string();
            }

            debug!("Using default config file path");
        }

        config_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&config_file_path)
            .unwrap_or_else(|err| {
                if err.kind() == ErrorKind::NotFound {
                    if config_file_path.eq(default_config_file.into_os_string().to_str().unwrap()) {
                        // Create config directory
                        if let Some(conf_dir) = Path::new(&config_file_path).parent() {
                            if !conf_dir.is_dir() {
                                DirBuilder::new().recursive(true).create(conf_dir).unwrap();
                            }
                        }

                        File::create(config_file_path)
                            .expect("Could not create default config file")
                    } else {
                        panic!("Could not find config file: {:?}", err);
                    }
                } else {
                    panic!("Could not open config file: {:?}", err);
                }
            });

        read_bytes = config_file
            .read_to_string(&mut config_string)
            .unwrap_or_else(|err| match err.kind() {
                ErrorKind::NotFound => {
                    error!("No config file: {}", err);
                    0
                }
                ErrorKind::Other => {
                    error!("Other config file err: {}", err);
                    0
                }
                _ => {
                    debug!("Read config file: No error");
                    0
                }
            });

        if read_bytes > 0 {
            // Temporary value dropped
            config = toml::from_str(config_string.trim()).unwrap();

            return Some(config);
        }

        info!("Config file is empty, using default config values");

        // Write default config to file
        config_file
            .write_all(
                toml::to_string(&self)
                    .unwrap_or_else(|e| panic!("blaaaa {:?}", e))
                    .as_bytes(),
            )
            .unwrap();
        debug!("Updated config file with config values");

        None
    }

    #[allow(dead_code)]
    pub fn update_config_file(self, config_file_path: &str) {
        let mut config_file: File;

        debug!("Config file path: {}", config_file_path);

        config_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(config_file_path)
            .unwrap();

        config_file
            .write_all(
                toml::to_string(&self)
                    .unwrap_or_else(|e| panic!("blaaaa {:?}", e))
                    .as_bytes(),
            )
            .unwrap();
        debug!("Updated config file with config values");
    }
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
