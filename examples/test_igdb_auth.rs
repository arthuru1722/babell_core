use babel_core::igdb::auth::get_access_token;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client_id = env::var("IGDB_CLIENT_ID")
        .unwrap_or_else(|_| "seu_client_id_temporario".to_string());
    let client_secret = env::var("IGDB_CLIENT_SECRET")
        .unwrap_or_else(|_| "seu_client_secret_temporario".to_string());

    println!("[TEST] Requesting Twitch access token...");

    match get_access_token(&client_id, &client_secret).await {
        Ok(token) => {
            println!("[TEST] Access token obtained successfully!");
            println!("Access Token: {}", token.access_token);
            println!("Expires In: {} seconds", token.expires_in);
            println!("Token Type: {}", token.token_type);
        }
        Err(e) => {
            eprintln!("[TEST] Failed to obtain access token: {}", e);
        }
    }

    Ok(())
}

