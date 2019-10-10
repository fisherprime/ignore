// SPDX-License-Identifier: MIT

extern crate chrono;
extern crate clap;
extern crate fern;

use clap::{App, Arg, ArgMatches};
use std::fs::File;
use std::io::ErrorKind;

const DEFAULT_CONFIG_FILE: &str = "~/.config/ignore-ng/config";

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
}

pub fn parse_flags() -> Result<ArgMatches<'static>, fern::InitError> {
    let matches = App::new("ignore-ng")
        .version(env!("CARGO_PKG_VERSION"))
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
