use super::models::SourceData;
use crate::utils::paths::get_base_directory;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub fn save_json(data: &SourceData) -> Result<PathBuf, Box<dyn Error>> {
    let mut path = get_base_directory()?;
    path.push("sources");
    path.push(&data.name);

    fs::create_dir_all(&path)?;

    let file_name = format!("{}.json", data.name);
    path.push(file_name);

    let json_string = serde_json::to_string_pretty(data)?;
    fs::write(&path, json_string)?;

    Ok(path)
}
