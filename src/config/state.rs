// SPDX-License-Identifier: MIT

//! The `state` module defines the last execution state's struct, trait & method implementations.

use std::error::Error as StdErr;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

/// Constant specifying the location suffix of the last run state file from some parent directory
/// (i.e.  system cache directory).
const STATE_FILE_PATH_SUFFIX: &str = "ignore/.state";

/// `struct` containing identifiers on the state of `ignore`'s last run.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct State {
    /// Absolute path to the state file (not for the user).
    #[serde(skip)]
    path: String,

    /// Timestamp of the last `ignore` execution.
    pub last_run: SystemTime,
}

/// [`std::Default`] trait implementation for [`config::State`].
impl Default for State {
    fn default() -> Self {
        Self {
            path: "".to_owned(),
            last_run: SystemTime::now(),
        }
    }
}

/// Method implementations for [`config::State`].
impl State {
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

    /// Saves the contents of the current [`config::State`] to the state file.
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
}
