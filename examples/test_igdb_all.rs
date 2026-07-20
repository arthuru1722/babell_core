use babel_core::igdb::dictionaries::{
    companies::sync_companies, genres::sync_genres, keywords::sync_keywords,
    language::sync_languages, themes::sync_themes, sources::sync_sources,
};
use babel_core::igdb::extra::core::{ExtraSyncMode, sync_extra};
use babel_core::igdb::games::core::sync_games;
use babel_core::igdb::search_improve::core::sync_search_improve;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client_id =
        env::var("IGDB_CLIENT_ID").unwrap_or_else(|_| "temporary_client_id".to_string());
    let access_token =
        env::var("IGDB_ACCESS_TOKEN").unwrap_or_else(|_| "temporary_access_token".to_string());

    sync_games(&client_id, &access_token).await?;
    sync_genres(&client_id, &access_token).await?;
    sync_keywords(&client_id, &access_token).await?;
    sync_companies(&client_id, &access_token).await?;
    sync_themes(&client_id, &access_token).await?;
    sync_languages(&client_id, &access_token).await?;
    sync_sources(&client_id, &access_token).await?;
    sync_search_improve(&client_id, &access_token).await?;
    sync_extra(&client_id, &access_token, ExtraSyncMode::All).await?;

    Ok(())
}
