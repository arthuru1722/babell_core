use super::models::IgdbSource; //aq tbm
use crate::igdb::pipeline::{DynError, run_sync_pipeline};

pub async fn sync_sources(
    //muda o nome daq
    client_id: &str,
    access_token: &str,
) -> Result<(), DynError> {
    //                        muda aq
    let table_sql = "CREATE TABLE IF NOT EXISTS sources (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        updated_at INTEGER NOT NULL
    );";
    //                                      muda aq
    let insert_sql = "INSERT INTO sources (id, name, updated_at) 
        VALUES (?1, ?2, ?3)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            updated_at = excluded.updated_at;";
    //                          aq tbm
    run_sync_pipeline::<IgdbSource, _>(
        client_id,
        access_token,
        "external_game_sources", //endpoint (muda tbm)
        "name, updated_at",
        "sources", // nome da tabela
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
