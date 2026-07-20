use super::models::IgdbLanguage;
use crate::igdb::pipeline::{DynError, run_sync_pipeline};

pub async fn sync_languages(client_id: &str, access_token: &str) -> Result<(), DynError> {
    //                       muda aq
    let table_sql = "CREATE TABLE IF NOT EXISTS languages (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        updated_at INTEGER NOT NULL
    );";
    //                                      muda aq
    let insert_sql = "INSERT INTO languages (id, name, updated_at) 
        VALUES (?1, ?2, ?3)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            updated_at = excluded.updated_at;";

    run_sync_pipeline::<IgdbLanguage, _>(
        client_id,
        access_token,
        "languages", //endpoint
        "name, updated_at",
        "languages", //nome da tabela
        table_sql,
        insert_sql,
        None,
        |item| {
            vec![
                Box::new(item.id),
                Box::new(item.name.clone()),
                Box::new(item.updated_at),
            ]
        },
    )
    .await?;

    Ok(())
}
