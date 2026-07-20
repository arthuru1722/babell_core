use super::models::IgdbTheme; //aq tbm
use crate::igdb::pipeline::{DynError, run_sync_pipeline};

pub async fn sync_themes(
    //muda o nome daq
    client_id: &str,
    access_token: &str,
) -> Result<(), DynError> {
    //                        muda aq
    let table_sql = "CREATE TABLE IF NOT EXISTS themes (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        slug TEXT NOT NULL,
        updated_at INTEGER NOT NULL
    );";
    //                                      muda aq
    let insert_sql = "INSERT INTO themes (id, name, slug, updated_at) 
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            slug = excluded.slug,
            updated_at = excluded.updated_at;";
    //                          aq tbm
    run_sync_pipeline::<IgdbTheme, _>(
        client_id,
        access_token,
        "themes", //endpoint (muda tbm)
        "name, slug, updated_at",
        "themes", // nome da tabela
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
