use directories::ProjectDirs;
use std::{error::Error, fs, path::PathBuf};

pub type DynError = Box<dyn Error + Send + Sync>;

pub fn get_base_directory() -> Result<PathBuf, Box<dyn Error>> {
    let proj_dirs =
        ProjectDirs::from("com", "Babell", "Babell").ok_or("Failed to determine OS directories")?;

    let path = proj_dirs.data_dir().to_path_buf();

    fs::create_dir_all(&path)?;

    Ok(path)
}
