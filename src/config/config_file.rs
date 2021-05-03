// SPDX-License-Identifier: MIT

//! The `config_file` module defines elements necessary for the setup and configuration of [`Config`]
//! (part of runtime environment).

use std::error::Error as StdErr;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Constant specifying the default gitignore template repo to use.
///
/// An alternative/supplement is: "https://github.com/toptal/gitignore" (gitignore.io)'s templates.
const GITIGNORE_DEFAULT_REPO: &str = "https://github.com/github/gitignore";

/// Constant specifying the repository cache subdirectory within the system's cache directory --for
/// storing gitignore template repositories--.
const GITIGNORE_REPO_CACHE_DIR: &str = "ignore/repos";

/// `struct` containing the runtime options parsed from a config file.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
#[serde(default)]
pub struct Config {
    /// Absolute path to the state file (not for the user).
    #[serde(skip)]
    config_path: String,

    /// Repository specific configuration options.
    pub repo_config: RepoConfig,
}

/// `struct` containing the config file's common repository options and an array of repository
/// specific runtime options.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct RepoConfig {
    /// Directory containing cached gitignore repositories.
    pub repo_cache_dir: String,

    /// [`RepoDetails`] for multiple template repositories.
    pub repo_details: Vec<RepoDetails>,
}

/// `struct` containing the config file's repository specific runtime options.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct RepoDetails {
    /// Choice of automatic (cached) repository updates.
    pub auto_update: bool,

    /// Choice of ignoring repository usage in `ignore`'s operations.
    pub ignore: bool,

    /// Relative path (to [`RepoConfig::repo_parent_dir`]) of gitignore template repository.
    pub repo_path: String,

    /// URL of git repository containing gitignore templates.
    pub repo_url: String,
}

impl Default for Config {
    fn default() -> Self {
        let default_gitignore_repo: String = GITIGNORE_DEFAULT_REPO.to_owned();
        let r_path: String;

        let mut r_cache_dir: PathBuf;

        let gitignore_repo_path = Path::new(&default_gitignore_repo);
        let mut gitignore_repo_path_components: Vec<_> = gitignore_repo_path
            .components()
            .map(|comp| comp.as_os_str())
            .collect();

        if gitignore_repo_path_components.len().lt(&2) {
            r_path = format!(
                "undefined/{}",
                gitignore_repo_path_components
                    .pop()
                    .unwrap()
                    .to_str()
                    .unwrap()
            );
        } else {
            r_path = format!(
                "{1}/{0}",
                gitignore_repo_path_components
                    .pop()
                    .unwrap()
                    .to_str()
                    .unwrap(),
                gitignore_repo_path_components
                    .pop()
                    .unwrap()
                    .to_str()
                    .unwrap()
            );
        }

        r_cache_dir = dirs_next::cache_dir().expect("Error obtaining system's cache directory");
        r_cache_dir.push(GITIGNORE_REPO_CACHE_DIR);

        Self {
            config_path: "".to_owned(),
            repo_config: RepoConfig {
                repo_cache_dir: r_cache_dir.into_os_string().into_string().unwrap(),
                repo_details: vec![RepoDetails {
                    auto_update: false,
                    ignore: false,
                    repo_url: default_gitignore_repo,
                    repo_path: r_path,
                }],
            },
        }
    }
}

/// Method implementations for [`Config`].
impl Config {
    /// Parses config file content & generates a [`Config`] item.
    pub fn parse(&mut self, config_file_path: &str) -> Result<Config, Box<dyn StdErr>> {
        use super::utils::create_file;

        debug!("Parsing config file");

        let mut config_file_content = String::new();

        if !Path::new(&config_file_path).exists() {
            create_file(&Path::new(&config_file_path))?;
        }

        let mut config_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(config_file_path)?;
        self.config_path = config_file_path.to_owned();

        if config_file
            .read_to_string(&mut config_file_content)
            .unwrap_or(0)
            > 0
        {
            match toml::from_str(config_file_content.trim()) {
                Ok(cfg) => {
                    let config = Config {
                        config_path: self.config_path.clone(),
                        ..cfg
                    };
                    debug!("Done parsing config file, config: {:#?}", config);

                    return Ok(config);
                }
                Err(_) => {
                    info!("Config file is invalid, backing up");
                    std::fs::copy(config_file_path, format!("{}.bak", config_file_path))?;
                    config_file.set_len(0)?;
                }
            }
        } else {
            info!("Config file is empty, using default config values");
        }

        self.update_file(&mut config_file)?;
        debug!("Config: {:#?}", self);

        Ok(self.clone())
    }

    /// Updates the content of the config file with the current [`Config`].
    fn update_file(&self, config_file: &mut File) -> Result<(), Box<dyn StdErr>> {
        config_file.write_all(toml::to_string(&self)?.as_bytes())?;
        debug!("Updated config file");

        Ok(())
    }

    /// Saves the content of the current [`Config`] to the config file.
    pub fn save_file(&self) -> Result<(), Box<dyn StdErr>> {
        debug!("Updating file: {}", self.config_path);

        let mut config_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.config_path)?;
        config_file.set_len(0)?;

        self.update_file(&mut config_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::utils::create_file;

    /**
     * Assert correctness of the default runtime options, includes the config.
     * TODO: add necessary fields
     */
    /*     #[test]
     *     fn option_parse_test() {
     *         let options = match Options::parse() {
     *             Some(val) => val,
     *             None => None,
     *         };
     *
     *         assert!(options);
     *     } */

    #[test]
    /// Assert correctness of the default config options.
    fn config_var_create_test() {
        let config = Config::default();

        let mut parent_dir = dirs_next::cache_dir().unwrap();
        parent_dir.push("ignore/repos");

        let test_config = Config {
            config_path: "".to_owned(),
            repo_config: RepoConfig {
                repo_cache_dir: parent_dir.into_os_string().into_string().unwrap(),
                repo_details: vec![RepoDetails {
                    auto_update: false,
                    ignore: false,
                    repo_url: GITIGNORE_DEFAULT_REPO.to_owned(),
                    repo_path: "github/gitignore".to_owned(),
                }],
            },
        };

        assert!(test_config.eq(&config));
    }

    #[test]
    /// Assert correctness of parsed default config file.
    fn config_file_parse_test() {
        let mut config = Config::default();

        let mut config_path = dirs_next::config_dir().unwrap();
        config_path.push("ignore/config.toml");

        // Create default config file should it not exist.
        if !Path::new(&config_path).exists() {
            create_file(&Path::new(&config_path)).unwrap();
        }

        // Parse default config file; populate `Config` variable with default default config on error (non-existent).
        config = config
            .parse(&config_path.clone().into_os_string().into_string().unwrap())
            .map(|cfg| cfg)
            .unwrap_or_else(|err| {
                error!("Config parse error, using the default: {}", err);
                config.clone()
            });

        // Parse current config file & assert is similar to the default.
        config
            .parse(&config_path.into_os_string().into_string().unwrap())
            .map(|cfg| assert!(cfg.eq(&config)))
            .unwrap_or_else(|err| panic!("Could not parse config: {}", err));
    }
}
