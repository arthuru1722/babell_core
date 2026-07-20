use babel_core::igdb::dictionaries::genres::sync_genres;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client_id = env::var("IGDB_CLIENT_ID")
        .unwrap_or_else(|_| "seu_client_id_temporario".to_string());
    let access_token = env::var("IGDB_ACCESS_TOKEN")
        .unwrap_or_else(|_| "seu_access_token_temporario".to_string());

    println!("[TEST] Iniciando sincronização do dicionário de Genres...");
    
    match sync_genres(&client_id, &access_token).await {
        Ok(()) => println!("[TEST] Dicionário de Gêneros sincronizado com sucesso!"),
        Err(e) => eprintln!("[TEST] Sincronização falhou: {}", e),
    }

    Ok(())
}