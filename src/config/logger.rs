// SPDX-License-Identifier: MIT

//! The `logger` module defines functions necessary for the setup of [`fern`].

use clap::ArgMatches;

/// Configures the [`fern`] logger.
///
/// This function configures the logger to output log messages using the `ISO` date format and
/// verbosity levels specified by the verbosity arguments (within [`clap::ArgMatches`]).
///
/// The arguments set the output verbosity for this crate to a maximum log level of either:
/// [`log::LevelFilter::Info`], [`log::LevelFilter::Debug`], [`log::LevelFilter::Trace`],
/// [`log::LevelFilter::Off`].
pub fn setup_logger(matches: &ArgMatches) -> Result<(), fern::InitError> {
    use fern::Dispatch;
    use log::LevelFilter;

    debug!("setting up logger");

    let mut verbose = true;

    let log_max_level = match matches.get_count("verbosity") {
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

    debug!("done setting up logger");

    Ok(())
}
