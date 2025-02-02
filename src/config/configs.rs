// SPDX-License-Identifier: MIT

//! The `config` module defines elements necessary for the setup and configuration of [`Config`]
//! (part of runtime environment).

use std::error::Error as StdErr;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Constant specifying the default gitignore template repo to use.
///
/// An alternative/supplement is: <https://github.com/toptal/gitignore> (gitignore.io)'s templates.
const GITIGNORE_DEFAULT_REPO: &str = "https://github.com/github/gitignore";

/// Constant specifying the repository cache subdirectory within the system's cache directory --for
/// storing gitignore template repositories--.
const GITIGNORE_REPO_CACHE_DIR: &str = "ignore/repos";

/// `struct` containing the runtime options loaded from a config file.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
#[serde(default)]
pub struct Config {
    /// Absolute path to the state file (not for the user).
    #[serde(skip)]
    config_path: String,

    /// Repository specific configuration options.
    pub repository: BaseRepoConfig,
}

/// `struct` containing the config file's common repository options and an array of repository
/// specific runtime options.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct BaseRepoConfig {
    /// Directory containing cached gitignore repositories.
    pub cache_dir: String,

    /// [`RepoConfig`] for multiple template repositories.
    pub config: Vec<RepoConfig>,
}

/// `struct` containing the config file's repository specific runtime options.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct RepoConfig {
    /// Choice of automatic (cached) repository updates.
    pub auto_update: bool,

    /// Choice of ignoring repository usage in `ignore`'s operations.
    pub skip: bool,

    /// Gitignore template's local cache directory relative to [`BaseRepoConfig::cache_dir`].
    pub path: String,

    /// URL of git repository containing gitignore templates.
    pub url: String,
}

impl Default for Config {
    fn default() -> Self {
        let default_gitignore_repo: String = GITIGNORE_DEFAULT_REPO.to_owned();

        let mut r_cache_dir: PathBuf;

        let gitignore_repo_path = Path::new(&default_gitignore_repo);
        let mut gitignore_repo_path_components: Vec<_> = gitignore_repo_path
            .components()
            .map(|comp| comp.as_os_str())
            .collect();

        let r_path: String = if gitignore_repo_path_components.len().lt(&2) {
            format!(
                "undefined/{}",
                gitignore_repo_path_components
                    .pop()
                    .unwrap()
                    .to_str()
                    .unwrap()
            )
        } else {
            format!(
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
            )
        };

        r_cache_dir =
            dirs_next::cache_dir().expect("dirs: failed to obtain system's cache directory");
        r_cache_dir.push(GITIGNORE_REPO_CACHE_DIR);

        Self {
            config_path: "".to_owned(),
            repository: BaseRepoConfig {
                cache_dir: r_cache_dir.into_os_string().into_string().unwrap(),
                config: vec![RepoConfig {
                    auto_update: false,
                    skip: false,
                    url: default_gitignore_repo,
                    path: r_path,
                }],
            },
        }
    }
}

/// Method implementations for [`Config`].
impl Config {
    /// Load config file content to generate the [`Config`] item.
    pub fn load(&mut self, config_file_path: &str) -> Result<(), Box<dyn StdErr>> {
        use crate::utils::create_file;

        debug!("config: file loading");

        if !Path::new(&config_file_path).exists() {
            create_file(Path::new(&config_file_path))?;
        }

        let mut config_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(config_file_path)?;
        config_file_path.clone_into(&mut self.config_path);

        let mut config_file_content = String::new();
        if config_file
            .read_to_string(&mut config_file_content)
            .unwrap_or(0)
            > 0
        {
            match toml::from_str(config_file_content.trim()) {
                Ok(cfg_content) => {
                    *self = Config {
                        config_path: self.config_path.clone(),
                        ..cfg_content
                    };
                    debug!("config: file loaded {:#?}", self);

                    return Ok(());
                }
                Err(_) => {
                    info!("config: invalid, backing up current config");
                    std::fs::copy(config_file_path, format!("{}.bak", config_file_path))?;
                    config_file.set_len(0)?;
                }
            }
        } else {
            // Assuming [`Config::default`] was called.
        }

        self.update_file(&mut config_file)?;
        debug!("config: final values {:#?}", self);

        Ok(())
    }

    /// Updates the content of the config file with the current [`Config`].
    fn update_file(&self, config_file: &mut File) -> Result<(), Box<dyn StdErr>> {
        config_file.write_all(toml::to_string(&self)?.as_bytes())?;
        debug!("config: file updated");

        Ok(())
    }

    /// Saves the content of the current [`Config`] to the config file.
    #[allow(dead_code)]
    pub fn save_file(&self) -> Result<(), Box<dyn StdErr>> {
        debug!("config: file updating {}", self.config_path);

        let mut config_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.config_path)?;

        self.update_file(&mut config_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::utils::create_file;

    /**
     * Assert correctness of the default runtime options, includes the config.
     * TODO: add necessary fields
     */
    /*     #[test]
     *     fn option_load_test() {
     *         let options = match RuntimeConfig::load() {
     *             Some(val) => val,
     *             None => None,
     *         };
     *
     *         assert!(options);
     *     } */

    #[test]
    /// Assert correctness of the default config options.
    fn config_variable_test() {
        let config = Config::default();

        let mut parent_dir = dirs_next::cache_dir().unwrap();
        parent_dir.push("ignore/repos");

        let test_config = Config {
            config_path: "".to_owned(),
            repository: BaseRepoConfig {
                cache_dir: parent_dir.into_os_string().into_string().unwrap(),
                config: vec![RepoConfig {
                    auto_update: false,
                    skip: false,
                    url: GITIGNORE_DEFAULT_REPO.to_owned(),
                    path: "github/gitignore".to_owned(),
                }],
            },
        };

        assert!(test_config.eq(&config));
    }

    // Useless.
    /*     #[test]
     *     /// Assert correctness of the loaded default config file.
     *     fn config_file_load_test() {
     *         let mut config_path = dirs_next::config_dir().unwrap();
     *         config_path.push("ignore/config.toml");
     *
     *         // Create default config file should it not exist.
     *         if !Path::new(&config_path).exists() {
     *             create_file(&Path::new(&config_path)).unwrap();
     *         }
     *
     *         // Parse default config file; populate `Config` variable with default default config on error (non-existent).
     *         let mut config = Config::default();
     *         config
     *             .load(&config_path.clone().into_os_string().into_string().unwrap())
     *             .unwrap();
     *
     *         assert!(config.eq(&Config::default()));
     *     } */
}
