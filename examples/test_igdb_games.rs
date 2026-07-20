use babel_core::igdb::games::core::sync_games;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client_id = env::var("IGDB_CLIENT_ID")
        .unwrap_or_else(|_| "seu_client_id_temporario".to_string());
    let access_token = env::var("IGDB_ACCESS_TOKEN")
        .unwrap_or_else(|_| "seu_access_token_temporario".to_string());

    println!("[TEST] Iniciando a sincronização da tabela principal de Games...");
    println!("[TEST] Filtro aplicado internamente: external_games.external_game_source = (1) & cover.url != null");
    
    match sync_games(&client_id, &access_token).await {
        Ok(()) => println!("[TEST] Tabela de Games sincronizada com sucesso no SQLite!"),
        Err(e) => eprintln!("[TEST] A sincronização de Games falhou: {}", e),
    }

    Ok(())
}