use super::models::IgdbGenre;
use crate::igdb::pipeline::{DynError, run_sync_pipeline};

pub async fn sync_genres(client_id: &str, access_token: &str) -> Result<(), DynError> {
    let table_sql = "CREATE TABLE IF NOT EXISTS genres (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        slug TEXT NOT NULL,
        updated_at INTEGER NOT NULL
    );";

    let insert_sql = "INSERT INTO genres (id, name, slug, updated_at) 
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            slug = excluded.slug,
            updated_at = excluded.updated_at;";

    run_sync_pipeline::<IgdbGenre, _>(
        client_id,
        access_token,
        "genres",
        "name, slug, updated_at",
        "genres",
        table_sql,
        insert_sql,
        None,
        |item| {
            vec![
                Box::new(item.id),
                Box::new(item.name.clone()),
                Box::new(item.slug.clone()),
                Box::new(item.updated_at),
            ]
        },
    )
    .await?;

    Ok(())
}
