use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SearchGame {
    pub game_id: i64,
    pub name: String,
    pub cover_url: Option<String>,
    pub release_date: Option<i64>,
    pub rating: Option<f64>,
    pub aggregated_rating: Option<f64>,
    pub summary: Option<String>,
    pub storyline: Option<String>,
    pub updated_at: i64,
}