use babel_core::igdb::dictionaries::keywords::sync_keywords;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client_id = env::var("IGDB_CLIENT_ID")
        .unwrap_or_else(|_| "seu_client_id_temporario".to_string());
    let access_token = env::var("IGDB_ACCESS_TOKEN")
        .unwrap_or_else(|_| "seu_access_token_temporario".to_string());

    println!("[TEST] Starting independent Keywords dictionary sync inside igdb/dictionaries...");
    
    match sync_keywords(&client_id, &access_token).await {
        Ok(()) => println!("[TEST] Keywords dictionary synchronized successfully!"),
        Err(e) => eprintln!("[TEST] Synchronization failed: {}", e),
    }

    Ok(())
}