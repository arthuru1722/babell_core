use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub(crate) struct FlareSolverrRequest {
    pub cmd: String,
    pub url: String,
    #[serde(rename = "maxTimeout")]
    pub max_timeout: u32,
}

#[derive(Deserialize, Debug)]
pub struct FlareSolverrRawResponse {
    pub status: String,
    pub solution: FlareSolverrSolution,
}

#[derive(Deserialize, Debug)]
pub struct FlareSolverrSolution {
    pub response: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Download {
    pub title: String,
    #[serde(rename = "fileSize")]
    pub file_size: String,
    pub uris: Vec<String>,
    #[serde(rename = "uploadDate")]
    pub upload_date: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SourceData {
    pub name: String,
    pub downloads: Vec<Download>,
}
