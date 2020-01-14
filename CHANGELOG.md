<a name=""></a>
##  (2020-01-14)


#### Features

* ***:**
  *  Add dirs & regex crates with initial code ([c3965e9b](c3965e9b))
  *  Add serde & toml crates ([cbc3f56b](cbc3f56b))
  *  Add git2 crate & initial code ([380d815c](380d815c))
  *  Add cargo files & define dependencies ([abac05bf](abac05bf))
* **.gitlab-ci.yml:**
  *  Add docs generation task ([545425a7](545425a7))
  *  Add CARGO_HOME variable ([788f4fb3](788f4fb3))
  *  Add Gitlab CI config ([4db6d864](4db6d864))
* **CHANGELOG.md:**  Add CHANGELOG ([38f844c6](38f844c6))
* **LICENSE.md:**  Add licence file ([1d44689c](1d44689c))
* **README.md:**  Add README ([6363f29c](6363f29c))
* **app.rs:**
  *  Add preliminary template deduplication ([b95da58e](b95da58e))
  *  Enable the config save function ([045289c7](045289c7))
  *  Add WIP ([25cb5f41](25cb5f41))
* **config.rs:**
  *  Add clap setting for required args ([991d5c11](991d5c11))
  *  Add RepoDetails ignore option ([30e6d0ef](30e6d0ef))
  *  Add config struct tests ([a3402bc6](a3402bc6))
  *  Add the Options struct ([3acbead2](3acbead2))
* **src/*.rs:**
  *  Add support for multiple template sources ([50b9ce9e](50b9ce9e))
  *  Add support for user & path templates ([dab1c82f](dab1c82f))
  *  Populate initial files with code ([0d1fbc57](0d1fbc57))

#### Bug Fixes

* **.gitlab-ci.yml:**
  *  Remove unnecessary build job ([b2b743b3](b2b743b3))
  *  Fix after_script spelling error ([cd8c0f3c](cd8c0f3c))
* **Cargo.toml:**  Correct erroneous version bump ([d108400d](d108400d))
* **app.rs:**
  *  Remove directories from template list ([1464c853](1464c853))
  *  Fix repository update function ([afd88a19](afd88a19))
  *  Fix consolidation file overwrite ([a1869487](a1869487))
* **config.rs:**
  *  Remove lifetimes & change to String ([c1dfd9d9](c1dfd9d9))
  *  Fix WIP linter warnings ([19f7d843](19f7d843))

#### Performance

* ***:**
  *  Change Config to a member of Options ([489f53c7](489f53c7))
  *  Move clap create (+macro) loading to root ([627f2345](627f2345))
  *  Move serde crate loading to config.rs ([202a40cc](202a40cc))
  *  Add Cargo.lock to .gitignore ([4cf2f01d](4cf2f01d))
* **Config.toml:**  Remove ref to unused crate ([e65a0407](e65a0407))
* **config.re:**  Comment out unused crate ([199ce723](199ce723))
* **config.rs:**
  *  Remove app_config cloning ([eeac5794](eeac5794))
  *  Rework functions into struct methods ([9241a390](9241a390))
  *  Section the Config struct ([d141d133](d141d133))
* **main.rs:**  Replace unwrap with expect ([cb3302d0](cb3302d0))
