// SPDX-License-Identifier: MIT

//! The `state` module defines the last execution [`State`]'s struct, its trait & method
//! implementations.

use std::error::Error as StdErr;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

/// [`u64`] constant specifying the amount of seconds in a day.
const SECONDS_IN_DAY: u64 = 60 * 60 * 24;

/// [`std::time::Duration`] constant specifying the time to consider a repository's contents as stale.
const REPO_UPDATE_LIMIT: Duration = Duration::from_secs(SECONDS_IN_DAY * 7);

/// Constant specifying the location suffix of the last run state file from some parent directory
/// (i.e.  system cache directory).
const STATE_FILE_PATH_SUFFIX: &str = "ignore/.state";

/// `struct` containing identifiers on the state of `ignore`'s last run.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct State {
    /// Absolute path to the state file (not for the user).
    #[serde(skip)]
    path: String,

    /// Timestamp of the last `ignore` `app::update_gitignore_repos` execution.
    pub last_update: SystemTime,
}

impl Default for State {
    fn default() -> Self {
        Self {
            path: "".to_owned(),
            last_update: SystemTime::now(),
        }
    }
}

/// Method implementations for [`State`].
impl State {
    /// Creates a new [`State`] from a provided [`SystemTime`] reference.
    pub fn new(now: &SystemTime) -> Self {
        Self {
            last_update: now.checked_sub(Duration::from_secs(1)).unwrap(),
            ..Default::default()
        }
    }

    /// Parses state file contents & generates a [`State`] item.
    // Passing a reference to Config struct avoid taking ownership.
    pub fn parse(&mut self) -> Result<State, Box<dyn StdErr>> {
        use super::utils::create_file;

        let mut state_file_path = dirs::cache_dir().unwrap();
        state_file_path.push(STATE_FILE_PATH_SUFFIX);

        let mut state_content = String::new();

        if !&state_file_path.exists() {
            create_file(&state_file_path)?;
        }

        let mut state_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(state_file_path.clone())?;
        self.path = state_file_path
            .into_os_string()
            .to_str()
            .unwrap()
            .to_owned();

        if state_file.read_to_string(&mut state_content).unwrap_or(0) > 0 {
            if let Ok(state) = toml::from_str(state_content.trim()) {
                let internal_state = State {
                    path: self.path.clone(),
                    ..state
                };
                debug!("Done parsing state file, state: {:#?}", internal_state);

                return Ok(internal_state);
            }
        }

        info!("State file is empty");
        self.update_file(&mut state_file)?;
        debug!("State: {:#?}", self);

        Ok(self.clone())
    }

    /// Updates the contents of the state file with the current [`State`].
    fn update_file(&self, state_file: &mut File) -> Result<(), Box<dyn StdErr>> {
        state_file.write_all(toml::to_string(&self)?.as_bytes())?;
        debug!("Updated state file");

        Ok(())
    }

    /// Saves the contents of the current [`State`] to the state file.
    pub fn save_file(&self) -> Result<(), Box<dyn StdErr>> {
        debug!("Updating file: {}", self.path);

        let mut state_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.path)?;
        state_file.set_len(0)?;

        self.update_file(&mut state_file)
    }

    /// Checks for staleness of the cached gitignore template repositories.
    ///
    /// This function compares the current [`SystemTime`] to the last repository update time.
    /// This function returns `true` (staleness state) if the time difference between now & the last
    /// repo update exceed [`REPO_UPDATE_LIMIT`], or the cache's ".state" file doesn't exist.
    /// Otherwise, this function returns` false`.
    pub fn check_staleness(&self, now: &SystemTime) -> Result<bool, Box<dyn StdErr>> {
        let last_update_duration = now.duration_since(self.last_update)?;
        let is_stale = { (last_update_duration > REPO_UPDATE_LIMIT) || now.eq(&self.last_update) };

        debug!(
            "Last repo update: {:#?}, now: {:#?}, difference: {:#?}, is stale: {}",
            self.last_update, now, last_update_duration, is_stale
        );

        Ok(is_stale)
    }
}
