// SPDX-License-Identifier: MIT

extern crate chrono;
extern crate clap;
extern crate fern;

use clap::{App, Arg, ArgMatches};

// TODO: populate this
#[allow(dead_code)]
fn read_config_file() {}

// TODO: populate this
#[allow(dead_code)]
pub fn get_args() {}

pub fn parse_flags() -> Result<ArgMatches<'static>, fern::InitError> {
    let matches = App::new("ignore-ng")
        .version("0.1.0")
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

    if let Err(err) = setup_logger(&matches) {
        return Err(err);
    }
    debug!("Logger is set up");

    Ok(matches)
}

pub fn setup_logger(matches: &ArgMatches) -> Result<(), fern::InitError> {
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
