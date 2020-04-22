// SPDX-License-Identifier: MIT

//! The `setup` module defines functions necessary for the setup of [`clap`] and [`fern`].

use clap::ArgMatches;

/// Configures [`clap`].
///
/// This function configures [`clap`] then calls [`clap::App::get_matches`] on the result to yield a
/// [`clap::ArgMatches`] item.
pub fn setup_clap(matches: &mut ArgMatches) {
    use clap::{App, AppSettings, Arg};

    // `env!("CARGO_PKG_VERSION")` replaced with `crate_version!`
    *matches = App::new("ignore")
            .setting(AppSettings::ArgRequiredElseHelp)
            .version(crate_version!())
            .about("A gitignore generator")
            .author("fisherprime")
            .arg(
                Arg::with_name("config")
                .help("Specify alternative config file to use.")
                .short("c")
                .long("config")
                .value_name("FILE")
                .takes_value(true)
            )
            .arg(
                Arg::with_name("list")
                .help("List all available languages, tools & projects.")
                .short("l")
                .long("list")
            )
            .arg(
                Arg::with_name("output")
                .help("Specify output filename, defaults to: gitignore.")
                .short("o")
                .long("output")
                .value_name("FILE")
                .takes_value(true)
            )
            .arg(
                Arg::with_name("template")
                .help("Case sensitive specification of language(s), tool(s) and/or project template(s) to use in generating gitignore.")
                .short("t")
                .long("templates")
                .value_name("TEMPLATE")
                .takes_value(true)
                .multiple(true)
            )
            .arg(
                Arg::with_name("update")
                .help("Manually update the gitignore template repo(s)")
                .short("u").long("update")
            )
            .arg(
                Arg::with_name("verbosity")
                .help("Set the level of verbosity for logs: -v or -vv.")
                .short("v")
                .long("verbose")
                .multiple(true)
            )
            .get_matches();
    debug!("Parsed command flags");
}

/// Configures the [`fern`] logger.
///
/// This function configures the logger to output log messages using the `ISO` date format and
/// verbosity levels specified by the verbosity arguments (within [`clap::ArgMatches`]).
/// The arguments set the output verbosity for this crate to a maximum log level of either:
/// [`log::LevelFilter::Info`], [`log::LevelFilter::Debug`], [`log::LevelFilter::Trace`],
/// [`log::LevelFilter::Off`].
pub fn setup_logger(matches: &ArgMatches) -> Result<(), fern::InitError> {
    use fern::Dispatch;
    use log::LevelFilter;

    debug!("Setting up logger");

    let mut verbose = true;

    let log_max_level = match matches.occurrences_of("verbosity") {
        0 => {
            verbose = false;
            LevelFilter::Info
        }
        1 => LevelFilter::Debug,
        2 => LevelFilter::Trace,
        _ => {
            println!("[WARN] Invalid verbosity level, defaulting to none");
            verbose = false;
            LevelFilter::Off
        }
    };

    if verbose {
        Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "{}[{}][{}] {}",
                    chrono::Local::now().format("[%Y-%m-%dT%H:%M:%S%z]"),
                    record.target(),
                    record.level(),
                    message
                ))
            })
            .level(log_max_level)
            .chain(std::io::stdout())
            // .chain(fern::log_file("output.log")?)
            .apply()?;
    } else {
        Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!("[{}] {}", record.level(), message))
            })
            .level(log_max_level)
            .chain(std::io::stdout())
            // .chain(fern::log_file("output.log")?)
            .apply()?;
    }

    debug!("Done setting up logger");

    Ok(())
}
