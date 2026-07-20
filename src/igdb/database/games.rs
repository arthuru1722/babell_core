use crate::igdb::database::models::SearchGame;
use crate::igdb::pipeline::DynError;
use crate::utils::paths::get_base_directory;

use rusqlite::{params, Connection};
use tokio::task;

/// Search games by name.
pub async fn search_games(
    query: &str,
    limit: usize,
    offset: usize,
) -> Result<Vec<SearchGame>, DynError> {
    let query = query.to_owned();

    task::spawn_blocking(move || -> Result<Vec<SearchGame>, DynError> {
        let db_path = get_base_directory()
            .map_err(|e| e.to_string())?
            .join("catalog")
            .join("igdb.db");

        let conn = Connection::open(db_path)?;

        let mut stmt = conn.prepare(
            "
            SELECT
                game_id,
                name,
                cover_url,
                release_date,
                rating,
                aggregated_rating,
                summary,
                storyline,
                updated_at
            FROM games
            WHERE name LIKE ?
            ORDER BY name
            LIMIT ?
            OFFSET ?
            ",
        )?;

        let games = stmt
            .query_map(
                params![format!("%{query}%"), limit as i64, offset as i64],
                |row| {
                    Ok(SearchGame {
                        game_id: row.get(0)?,
                        name: row.get(1)?,
                        cover_url: row.get(2)?,
                        release_date: row.get(3)?,
                        rating: row.get(4)?,
                        aggregated_rating: row.get(5)?,
                        summary: row.get(6)?,
                        storyline: row.get(7)?,
                        updated_at: row.get(8)?,
                    })
                },
            )?
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;

        Ok(games)
    })
    .await?
}

/// List the catalog without filtering.
pub async fn list_games(
    limit: usize,
    offset: usize,
) -> Result<Vec<SearchGame>, DynError> {
    task::spawn_blocking(move || -> Result<Vec<SearchGame>, DynError> {
        let db_path = get_base_directory()
            .map_err(|e| e.to_string())?
            .join("catalog")
            .join("igdb.db");

        let conn = Connection::open(db_path)?;

        let mut stmt = conn.prepare(
            "
            SELECT
                game_id,
                name,
                cover_url,
                release_date,
                rating,
                aggregated_rating,
                summary,
                storyline,
                updated_at
            FROM games
            ORDER BY name
            LIMIT ?
            OFFSET ?
            ",
        )?;

        let games = stmt
            .query_map(params![limit as i64, offset as i64], |row| {
                Ok(SearchGame {
                    game_id: row.get(0)?,
                    name: row.get(1)?,
                    cover_url: row.get(2)?,
                    release_date: row.get(3)?,
                    rating: row.get(4)?,
                    aggregated_rating: row.get(5)?,
                    summary: row.get(6)?,
                    storyline: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;

        Ok(games)
    })
    .await?
}