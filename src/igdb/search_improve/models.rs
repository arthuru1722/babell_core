use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct IgdbGameSearchImprove {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub genres: Vec<i64>,
    #[serde(default)]
    pub keywords: Vec<i64>,
    #[serde(default)]
    pub language_supports: Vec<IgdbLanguageSupport>,
    #[serde(default)]
    pub themes: Vec<i64>,
    #[serde(default)]
    pub involved_companies: Vec<IgdbInvolvedCompany>,
    #[serde(default)]
    pub alternative_names: Vec<IgdbAlternativeName>,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct IgdbLanguageSupport {
    pub id: i64,
    pub language: i64,
    pub language_support_type: i64,
}

#[derive(Debug, Deserialize)]
pub struct IgdbInvolvedCompany {
    pub id: i64,
    pub company: i64,
}

#[derive(Debug, Deserialize)]
pub struct IgdbAlternativeName {
    pub id: i64,
    pub name: String,
}
