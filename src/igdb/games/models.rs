use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IgdbGameCover {
    pub id: i64,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IgdbGame {
    pub id: i64,
    pub name: String,
    pub cover: Option<IgdbGameCover>,
    pub first_release_date: Option<i64>,
    pub rating: Option<f64>,
    pub aggregated_rating: Option<f64>,
    pub updated_at: i64,
}
