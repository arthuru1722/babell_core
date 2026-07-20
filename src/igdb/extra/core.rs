use super::models::{IgdbExternalGame, IgdbGameExtra, IgdbUrl, IgdbVideo};
use crate::igdb::pipeline::{DynError, add_column_if_missing, run_id_sync_pipeline};
use crate::utils::paths::get_base_directory;
use rusqlite::{Connection, Transaction, params};

const EXTRA_FIELDS: &str = "name, screenshots.url, artworks.url, summary, storyline, external_games.external_game_source, websites.url, videos.video_id, updated_at";

pub enum ExtraSyncMode {
    All,
    Game(i64),
}

pub async fn sync_extra(
    client_id: &str,
    access_token: &str,
    mode: ExtraSyncMode,
) -> Result<(), DynError> {
    let base_dir = get_base_directory().map_err(|e| e.to_string())?;
    let db_path = base_dir.join("catalog").join("igdb.db");
    std::fs::create_dir_all(db_path.parent().expect("database path has a parent"))?;

    let game_ids = {
        let conn = Connection::open(&db_path)?;
        initialize_tables(&conn)?;

        match mode {
            ExtraSyncMode::All => load_pending_game_ids(&conn)?,
            ExtraSyncMode::Game(game_id) => vec![game_id],
        }
    };

    if game_ids.is_empty() {
        println!("[SYNC] No games require extra data.");
        return Ok(());
    }

    println!(
        "[SYNC] Found {} games requiring extra data.",
        game_ids.len()
    );

    run_id_sync_pipeline(
        client_id,
        access_token,
        "games",
        EXTRA_FIELDS,
        game_ids,
        db_path,
        "extra game records",
        save_game_extras,
    )
    .await
}

fn initialize_tables(conn: &Connection) -> Result<(), DynError> {
    add_column_if_missing(conn, "games", "summary", "TEXT")?;
    add_column_if_missing(conn, "games", "storyline", "TEXT")?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS extra_games (
            game_id INTEGER PRIMARY KEY,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (game_id) REFERENCES games(game_id)
        );
        CREATE TABLE IF NOT EXISTS game_screenshots (
            game_id INTEGER NOT NULL,
            screenshot_id INTEGER NOT NULL,
            url TEXT NOT NULL,
            PRIMARY KEY (game_id, screenshot_id),
            FOREIGN KEY (game_id) REFERENCES games(game_id)
        );
        CREATE TABLE IF NOT EXISTS game_artworks (
            game_id INTEGER NOT NULL,
            artwork_id INTEGER NOT NULL,
            url TEXT NOT NULL,
            PRIMARY KEY (game_id, artwork_id),
            FOREIGN KEY (game_id) REFERENCES games(game_id)
        );
        CREATE TABLE IF NOT EXISTS game_external_games (
            game_id INTEGER NOT NULL,
            external_game_id INTEGER NOT NULL,
            external_game_source_id INTEGER NOT NULL,
            PRIMARY KEY (game_id, external_game_id),
            FOREIGN KEY (game_id) REFERENCES games(game_id)
        );
        CREATE TABLE IF NOT EXISTS game_websites (
            game_id INTEGER NOT NULL,
            website_id INTEGER NOT NULL,
            url TEXT NOT NULL,
            PRIMARY KEY (game_id, website_id),
            FOREIGN KEY (game_id) REFERENCES games(game_id)
        );
        CREATE TABLE IF NOT EXISTS game_videos (
            game_id INTEGER NOT NULL,
            igdb_video_id INTEGER NOT NULL,
            video_id TEXT NOT NULL,
            PRIMARY KEY (game_id, igdb_video_id),
            FOREIGN KEY (game_id) REFERENCES games(game_id)
        );",
    )?;

    Ok(())
}

fn load_pending_game_ids(conn: &Connection) -> Result<Vec<i64>, DynError> {
    let mut statement = conn.prepare(
        "SELECT games.game_id
         FROM games
         LEFT JOIN extra_games ON extra_games.game_id = games.game_id
         WHERE extra_games.updated_at IS NULL
            OR games.updated_at > extra_games.updated_at
         ORDER BY games.game_id",
    )?;
    let game_ids = statement
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<i64>, _>>()?;

    Ok(game_ids)
}

fn save_game_extras(conn: &mut Connection, games: &[IgdbGameExtra]) -> Result<(), DynError> {
    let transaction = conn.transaction()?;

    for game in games {
        replace_game_extra(&transaction, game)?;
    }

    transaction.commit()?;
    Ok(())
}

fn replace_game_extra(transaction: &Transaction<'_>, game: &IgdbGameExtra) -> Result<(), DynError> {
    transaction.execute(
        "UPDATE games SET summary = ?2, storyline = ?3 WHERE game_id = ?1",
        params![game.id, game.summary, game.storyline],
    )?;
    transaction.execute(
        "INSERT INTO extra_games (game_id, updated_at) VALUES (?1, ?2)
         ON CONFLICT(game_id) DO UPDATE SET updated_at = excluded.updated_at",
        params![game.id, game.updated_at],
    )?;

    replace_urls(
        transaction,
        "game_screenshots",
        "screenshot_id",
        game.id,
        &game.screenshots,
    )?;
    replace_urls(
        transaction,
        "game_artworks",
        "artwork_id",
        game.id,
        &game.artworks,
    )?;
    replace_external_games(transaction, game.id, &game.external_games)?;
    replace_urls(
        transaction,
        "game_websites",
        "website_id",
        game.id,
        &game.websites,
    )?;
    replace_videos(transaction, game.id, &game.videos)?;

    Ok(())
}

fn replace_urls(
    transaction: &Transaction<'_>,
    table_name: &str,
    id_column: &str,
    game_id: i64,
    values: &[IgdbUrl],
) -> Result<(), DynError> {
    transaction.execute(
        &format!("DELETE FROM {table_name} WHERE game_id = ?1"),
        [game_id],
    )?;

    let sql = format!("INSERT INTO {table_name} (game_id, {id_column}, url) VALUES (?1, ?2, ?3)");
    let mut statement = transaction.prepare_cached(&sql)?;
    for value in values {
        statement.execute(params![game_id, value.id, value.url])?;
    }

    Ok(())
}

fn replace_external_games(
    transaction: &Transaction<'_>,
    game_id: i64,
    values: &[IgdbExternalGame],
) -> Result<(), DynError> {
    transaction.execute(
        "DELETE FROM game_external_games WHERE game_id = ?1",
        [game_id],
    )?;

    let mut statement = transaction.prepare_cached(
        "INSERT INTO game_external_games (game_id, external_game_id, external_game_source_id)
         VALUES (?1, ?2, ?3)",
    )?;
    for value in values {
        statement.execute(params![game_id, value.id, value.external_game_source])?;
    }

    Ok(())
}

fn replace_videos(
    transaction: &Transaction<'_>,
    game_id: i64,
    values: &[IgdbVideo],
) -> Result<(), DynError> {
    transaction.execute("DELETE FROM game_videos WHERE game_id = ?1", [game_id])?;

    let mut statement = transaction.prepare_cached(
        "INSERT INTO game_videos (game_id, igdb_video_id, video_id) VALUES (?1, ?2, ?3)",
    )?;
    for value in values {
        statement.execute(params![game_id, value.id, value.video_id])?;
    }

    Ok(())
}
