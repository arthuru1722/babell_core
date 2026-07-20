pub(crate) mod client;
pub(crate) mod models;
pub(crate) mod parser;
pub(crate) mod storage;

pub use client::request_url;
pub use models::SourceData;
pub use parser::extract_json;
pub use storage::save_json;

pub async fn download_and_save_source(
    flaresolverr_url: &str,
    target_url: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let html = request_url(flaresolverr_url, target_url).await?;
    let data = extract_json(&html)?;

    save_json(&data)
}
