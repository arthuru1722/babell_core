use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IgdbGenre {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IgdbKeyword {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IgdbCompany {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IgdbTheme {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IgdbLanguage {
    pub id: i64,
    pub name: String,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IgdbSource {
    pub id: i64,
    pub name: String,
    pub updated_at: i64,
}