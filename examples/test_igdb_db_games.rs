use babel_core::igdb::database::games::search_games;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("[TEST] Searching games...");

    match search_games("resident evil 5", 20, 0).await {
        Ok(games) => {
            println!("[TEST] Found {} game(s):", games.len());

            for game in games {
                println!("------------------------------");
                println!("ID: {}", game.game_id);
                println!("Name: {}", game.name);
                println!("Cover: {:?}", game.cover_url);
                println!("Release Date: {:?}", game.release_date);
                println!("Rating: {:?}", game.rating);
                println!("Aggregated Rating: {:?}", game.aggregated_rating);
                println!("Summary: {:?}", game.summary);
                println!("Storyline: {:?}", game.storyline);
                println!("Updated At: {}", game.updated_at);
            }
        }
        Err(e) => {
            eprintln!("[TEST] Search failed: {}", e);
        }
    }

    Ok(())
}