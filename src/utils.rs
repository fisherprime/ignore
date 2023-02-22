// SPDX-License-Identifier: MIT

//! The `utils` module defines utility functions common to modules of the config` module.

use std::error::Error as StdErr;
use std::fs::File;
use std::path::Path;

/// Creates a file defined by a filepath.
///
/// This function builds a file path's directory hierarchy (if necessary) then creates the file
/// specified by the path.
pub fn create_file(file_path: &Path) -> Result<(), Box<dyn StdErr>> {
    use std::fs::DirBuilder;

    info!("creating file: {}", file_path.display());

    let file_dir = Path::new(&file_path).parent().unwrap();
    if !file_dir.is_dir() {
        DirBuilder::new().recursive(true).create(file_dir)?
    }

    File::create(file_path)?;

    Ok(())
}
