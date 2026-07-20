use super::models::SourceData;
use scraper::{Html, Selector};
use std::error::Error;

pub fn extract_json(html_content: &str) -> Result<SourceData, Box<dyn Error>> {
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("pre").map_err(|_| "Failed to parse HTML selector")?;

    let pre_element = document
        .select(&selector)
        .next()
        .ok_or("Pre element containing JSON not found")?;

    let raw_json = pre_element.text().collect::<String>();
    let data: SourceData = serde_json::from_str(&raw_json)?;

    Ok(data)
}
