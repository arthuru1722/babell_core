use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct IgdbGameExtra {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub screenshots: Vec<IgdbUrl>,
    #[serde(default)]
    pub artworks: Vec<IgdbUrl>,
    #[serde(default)]
    pub external_games: Vec<IgdbExternalGame>,
    #[serde(default)]
    pub websites: Vec<IgdbUrl>,
    #[serde(default)]
    pub videos: Vec<IgdbVideo>,
    pub summary: Option<String>,
    pub storyline: Option<String>,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct IgdbUrl {
    pub id: i64,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct IgdbExternalGame {
    pub id: i64,
    pub external_game_source: i64,
}

#[derive(Debug, Deserialize)]
pub struct IgdbVideo {
    pub id: i64,
    pub video_id: String,
}
