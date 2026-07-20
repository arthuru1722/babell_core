use super::models::IgdbGame;
use crate::igdb::pipeline::{DynError, add_column_if_missing, run_sync_pipeline};

pub async fn sync_games(client_id: &str, access_token: &str) -> Result<(), DynError> {
    let table_sql = "CREATE TABLE IF NOT EXISTS games (
        game_id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        cover_url TEXT,
        release_date INTEGER,
        rating REAL,
        aggregated_rating REAL,
        summary TEXT,
        storyline TEXT,
        updated_at INTEGER NOT NULL
    );";

    let base_dir = crate::utils::paths::get_base_directory().map_err(|e| e.to_string())?;
    let db_path = base_dir.join("catalog").join("igdb.db");
    std::fs::create_dir_all(db_path.parent().expect("database path has a parent"))?;
    let conn = rusqlite::Connection::open(db_path)?;
    conn.execute(table_sql, [])?;
    add_column_if_missing(&conn, "games", "summary", "TEXT")?;
    add_column_if_missing(&conn, "games", "storyline", "TEXT")?;

    let insert_sql = "INSERT INTO games (game_id, name, cover_url, release_date, rating, aggregated_rating, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) 
        ON CONFLICT(game_id) DO UPDATE SET
            name = excluded.name,
            cover_url = excluded.cover_url,
            release_date = excluded.release_date,
            rating = excluded.rating,
            aggregated_rating = excluded.aggregated_rating,
            updated_at = excluded.updated_at;";

    let extra_where =
        "external_games.external_game_source = (1, 11) & cover.image_id != null & game_type = (0,8,9)";

    run_sync_pipeline::<IgdbGame, _>(
        client_id,
        access_token,
        "games",
        "name, cover.image_id, first_release_date, rating, aggregated_rating, updated_at",
        "games",
        table_sql,
        insert_sql,
        Some(extra_where),
        |item| {
            vec![
                Box::new(item.id),
                Box::new(item.name.clone()),
                Box::new(item.cover.as_ref().map(|c| c.image_id.clone())),
                Box::new(item.first_release_date),
                Box::new(item.rating),
                Box::new(item.aggregated_rating),
                Box::new(item.updated_at),
            ]
        },
    )
    .await?;

    Ok(())
}