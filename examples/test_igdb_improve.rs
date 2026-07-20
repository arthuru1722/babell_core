use babel_core::igdb::search_improve::core::sync_search_improve;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client_id =
        env::var("IGDB_CLIENT_ID").unwrap_or_else(|_| "temporary_client_id".to_string());
    let access_token =
        env::var("IGDB_ACCESS_TOKEN").unwrap_or_else(|_| "temporary_access_token".to_string());

    sync_search_improve(&client_id, &access_token).await
}
