use super::models::IgdbGameSearchImprove;
use crate::igdb::pipeline::{DynError, add_column_if_missing, run_id_sync_pipeline};
use crate::utils::paths::get_base_directory;
use rusqlite::{Connection, Transaction, params};

pub async fn sync_search_improve(client_id: &str, access_token: &str) -> Result<(), DynError> {
    let base_dir = get_base_directory().map_err(|e| e.to_string())?;
    let db_path = base_dir.join("catalog").join("igdb.db");
    std::fs::create_dir_all(db_path.parent().expect("database path has a parent"))?;

    let game_ids = {
        let conn = Connection::open(&db_path)?;
        initialize_tables(&conn)?;
        load_pending_game_ids(&conn)?
    };

    if game_ids.is_empty() {
        println!("[SYNC] No games require additional data.");
        return Ok(());
    }

    println!(
        "[SYNC] Found {} games requiring additional data.",
        game_ids.len()
    );

    run_id_sync_pipeline(
        client_id,
        access_token,
        "games",
        "name, genres, keywords, language_supports.language, language_supports.language_support_type, themes, involved_companies.company, alternative_names.name, updated_at",
        game_ids,
        db_path,
        "additional game records",
        save_game_relations,
    )
    .await
}

fn initialize_tables(conn: &Connection) -> Result<(), DynError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS search_improve_games (
            game_id INTEGER PRIMARY KEY,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (game_id) REFERENCES games(game_id)
        );
        CREATE TABLE IF NOT EXISTS game_genres (
            game_id INTEGER NOT NULL,
            genre_id INTEGER NOT NULL,
            PRIMARY KEY (game_id, genre_id),
            FOREIGN KEY (game_id) REFERENCES games(game_id)
        );
        CREATE TABLE IF NOT EXISTS game_keywords (
            game_id INTEGER NOT NULL,
            keyword_id INTEGER NOT NULL,
            PRIMARY KEY (game_id, keyword_id),
            FOREIGN KEY (game_id) REFERENCES games(game_id)
        );
        CREATE TABLE IF NOT EXISTS game_language_supports (
            game_id INTEGER NOT NULL,
            language_support_id INTEGER NOT NULL,
            language_id INTEGER NOT NULL,
            language_support_type_id INTEGER NOT NULL,
            PRIMARY KEY (game_id, language_support_id),
            FOREIGN KEY (game_id) REFERENCES games(game_id)
        );
        CREATE TABLE IF NOT EXISTS game_themes (
            game_id INTEGER NOT NULL,
            theme_id INTEGER NOT NULL,
            PRIMARY KEY (game_id, theme_id),
            FOREIGN KEY (game_id) REFERENCES games(game_id)
        );
        CREATE TABLE IF NOT EXISTS game_involved_companies (
            game_id INTEGER NOT NULL,
            involved_company_id INTEGER NOT NULL,
            company_id INTEGER NOT NULL,
            PRIMARY KEY (game_id, involved_company_id),
            FOREIGN KEY (game_id) REFERENCES games(game_id)
        );
        CREATE TABLE IF NOT EXISTS game_alternative_names (
            game_id INTEGER NOT NULL,
            alternative_name_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            PRIMARY KEY (game_id, alternative_name_id),
            FOREIGN KEY (game_id) REFERENCES games(game_id)
        );",
    )?;

    let language_id_added =
        add_column_if_missing(conn, "game_language_supports", "language_id", "INTEGER")?;
    let language_support_type_id_added = add_column_if_missing(
        conn,
        "game_language_supports",
        "language_support_type_id",
        "INTEGER",
    )?;
    let language_supports_changed = language_id_added || language_support_type_id_added;
    let involved_companies_changed =
        add_column_if_missing(conn, "game_involved_companies", "company_id", "INTEGER")?;
    let alternative_names_changed =
        add_column_if_missing(conn, "game_alternative_names", "name", "TEXT")?;

    if language_supports_changed || involved_companies_changed || alternative_names_changed {
        conn.execute("UPDATE search_improve_games SET updated_at = 0", [])?;
    }

    Ok(())
}

fn load_pending_game_ids(conn: &Connection) -> Result<Vec<i64>, DynError> {
    let mut statement = conn.prepare(
        "SELECT games.game_id
         FROM games
         LEFT JOIN search_improve_games ON search_improve_games.game_id = games.game_id
         WHERE search_improve_games.updated_at IS NULL
            OR games.updated_at > search_improve_games.updated_at
         ORDER BY games.game_id",
    )?;
    let game_ids = statement
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<i64>, _>>()?;

    Ok(game_ids)
}

fn save_game_relations(
    conn: &mut Connection,
    games: &[IgdbGameSearchImprove],
) -> Result<(), DynError> {
    let transaction = conn.transaction()?;

    for game in games {
        replace_game_relations(&transaction, game)?;
    }

    transaction.commit()?;
    Ok(())
}

fn replace_game_relations(
    transaction: &Transaction<'_>,
    game: &IgdbGameSearchImprove,
) -> Result<(), DynError> {
    transaction.execute(
        "INSERT INTO search_improve_games (game_id, updated_at) VALUES (?1, ?2)
         ON CONFLICT(game_id) DO UPDATE SET updated_at = excluded.updated_at",
        params![game.id, game.updated_at],
    )?;

    replace_relations(
        transaction,
        "game_genres",
        "genre_id",
        game.id,
        &game.genres,
    )?;
    replace_relations(
        transaction,
        "game_keywords",
        "keyword_id",
        game.id,
        &game.keywords,
    )?;
    replace_language_supports(transaction, game.id, &game.language_supports)?;
    replace_relations(
        transaction,
        "game_themes",
        "theme_id",
        game.id,
        &game.themes,
    )?;
    replace_involved_companies(transaction, game.id, &game.involved_companies)?;
    replace_alternative_names(transaction, game.id, &game.alternative_names)?;

    Ok(())
}

fn replace_alternative_names(
    transaction: &Transaction<'_>,
    game_id: i64,
    alternative_names: &[super::models::IgdbAlternativeName],
) -> Result<(), DynError> {
    transaction.execute(
        "DELETE FROM game_alternative_names WHERE game_id = ?1",
        [game_id],
    )?;

    let mut statement = transaction.prepare_cached(
        "INSERT INTO game_alternative_names (game_id, alternative_name_id, name)
         VALUES (?1, ?2, ?3)",
    )?;
    for alternative_name in alternative_names {
        statement.execute(params![game_id, alternative_name.id, alternative_name.name])?;
    }

    Ok(())
}

fn replace_language_supports(
    transaction: &Transaction<'_>,
    game_id: i64,
    language_supports: &[super::models::IgdbLanguageSupport],
) -> Result<(), DynError> {
    transaction.execute(
        "DELETE FROM game_language_supports WHERE game_id = ?1",
        [game_id],
    )?;

    let mut statement = transaction.prepare_cached(
        "INSERT INTO game_language_supports (game_id, language_support_id, language_id, language_support_type_id)
         VALUES (?1, ?2, ?3, ?4)",
    )?;
    for language_support in language_supports {
        statement.execute(params![
            game_id,
            language_support.id,
            language_support.language,
            language_support.language_support_type,
        ])?;
    }

    Ok(())
}

fn replace_involved_companies(
    transaction: &Transaction<'_>,
    game_id: i64,
    involved_companies: &[super::models::IgdbInvolvedCompany],
) -> Result<(), DynError> {
    transaction.execute(
        "DELETE FROM game_involved_companies WHERE game_id = ?1",
        [game_id],
    )?;

    let mut statement = transaction.prepare_cached(
        "INSERT INTO game_involved_companies (game_id, involved_company_id, company_id)
         VALUES (?1, ?2, ?3)",
    )?;
    for involved_company in involved_companies {
        statement.execute(params![
            game_id,
            involved_company.id,
            involved_company.company
        ])?;
    }

    Ok(())
}

fn replace_relations(
    transaction: &Transaction<'_>,
    table_name: &str,
    relation_column: &str,
    game_id: i64,
    relation_ids: &[i64],
) -> Result<(), DynError> {
    transaction.execute(
        &format!("DELETE FROM {table_name} WHERE game_id = ?1"),
        [game_id],
    )?;

    let insert_sql =
        format!("INSERT INTO {table_name} (game_id, {relation_column}) VALUES (?1, ?2)");
    let mut statement = transaction.prepare_cached(&insert_sql)?;
    for relation_id in relation_ids {
        statement.execute(params![game_id, relation_id])?;
    }

    Ok(())
}
