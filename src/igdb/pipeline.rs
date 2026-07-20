use crate::utils::paths::get_base_directory;
use futures::StreamExt;
use rusqlite::{Connection, ToSql};
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub type DynError = Box<dyn Error + Send + Sync>;

const API_BATCH_SIZE: usize = 500;
const REQUEST_CONCURRENCY: usize = 4;
const DATABASE_BATCH_SIZE: usize = 5_000;

pub fn add_column_if_missing(
    conn: &Connection,
    table_name: &str,
    column_name: &str,
    column_definition: &str,
) -> Result<bool, DynError> {
    let mut statement = conn.prepare(&format!("PRAGMA table_info({table_name})"))?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;

    if columns.iter().any(|column| column == column_name) {
        return Ok(false);
    }

    conn.execute(
        &format!("ALTER TABLE {table_name} ADD COLUMN {column_name} {column_definition}"),
        [],
    )?;
    Ok(true)
}

pub async fn run_sync_pipeline<T, F>(
    client_id: &str,
    access_token: &str,
    endpoint: &'static str,
    fields: &'static str,
    table_name: &'static str,
    table_init_sql: &'static str,
    insert_sql: &'static str,
    extra_where: Option<&str>,
    params_fn: F,
) -> Result<(), DynError>
where
    T: serde::de::DeserializeOwned + Send + 'static,
    F: Fn(&T) -> Vec<Box<dyn ToSql + Send>> + Copy + Send + Sync + 'static,
{
    let total_start = Instant::now();
    println!("[START] Starting synchronization for '{table_name}'.");

    let base_dir = get_base_directory().map_err(|e| e.to_string())?;
    let db_path = base_dir.join("catalog").join("igdb.db");
    std::fs::create_dir_all(db_path.parent().expect("database path has a parent"))?;

    {
        let conn = Connection::open(&db_path)?;
        conn.execute(table_init_sql, [])?;
    }

    let start_timestamp = {
        let conn = Connection::open(&db_path)?;
        let query = format!("SELECT MAX(updated_at) FROM {table_name}");
        match conn.query_row(&query, [], |row| row.get::<_, i64>(0)) {
            Ok(timestamp) => {
                let margin_timestamp = std::cmp::max(0, timestamp - 100);
                println!("[SYNC] Existing data found. Using a 100-second margin.");
                Some(margin_timestamp)
            }
            Err(_) => {
                println!("[SYNC] No existing data found. Starting from the beginning.");
                None
            }
        }
    };

    let (sender, receiver) = mpsc::channel::<Vec<T>>(32);
    let http_client = Arc::new(build_igdb_client(client_id, access_token)?);
    let extra_where = extra_where.map(str::to_owned);

    let fetch_task = tokio::spawn(async move {
        let should_stop = Arc::new(AtomicBool::new(false));
        let mut current_offset = 0_u32;

        loop {
            if should_stop.load(Ordering::Relaxed) {
                break;
            }

            let mut offsets = Vec::with_capacity(REQUEST_CONCURRENCY);
            for _ in 0..REQUEST_CONCURRENCY {
                offsets.push(current_offset);
                current_offset = match current_offset.checked_add(API_BATCH_SIZE as u32) {
                    Some(next) => next,
                    None => {
                        should_stop.store(true, Ordering::Relaxed);
                        break;
                    }
                };
            }

            if offsets.is_empty() {
                break;
            }

            let mut workers = futures::stream::iter(offsets)
                .map(|offset| {
                    let client = http_client.clone();
                    let stop_flag = should_stop.clone();
                    let extra_where = extra_where.clone();
                    async move {
                        if stop_flag.load(Ordering::Relaxed) {
                            return (offset, None, Instant::now().elapsed());
                        }

                        let mut conditions = Vec::new();
                        if let Some(timestamp) = start_timestamp {
                            conditions.push(format!("updated_at >= {timestamp}"));
                        }
                        if let Some(extra_where) = extra_where {
                            conditions.push(extra_where);
                        }

                        let where_clause = if conditions.is_empty() {
                            String::new()
                        } else {
                            format!("where {}; ", conditions.join(" & "))
                        };
                        let query = format!(
                            "fields {fields}; {where_clause}sort updated_at asc; limit {API_BATCH_SIZE}; offset {offset};"
                        );

                        let request_start = Instant::now();
                        let response = client
                            .post(format!("https://api.igdb.com/v4/{endpoint}"))
                            .body(query)
                            .send()
                            .await;
                        (offset, Some(response), request_start.elapsed())
                    }
                })
                .buffer_unordered(REQUEST_CONCURRENCY);

            let mut received_empty_batch = false;

            while let Some((offset, response, request_duration)) = workers.next().await {
                let Some(response) = response else {
                    continue;
                };
                println!(
                    "[API] Request to '{endpoint}' at offset {offset} completed in {request_duration:.2?}."
                );

                let response = match response {
                    Ok(response) => response,
                    Err(error) => {
                        eprintln!("[NETWORK] Offset {offset}: {error}");
                        continue;
                    }
                };

                if !response.status().is_success() {
                    eprintln!("[API] Offset {offset} returned {}.", response.status());
                    continue;
                }

                let items: Vec<T> = match response.json().await {
                    Ok(items) => items,
                    Err(error) => {
                        eprintln!("[PARSE] Offset {offset}: {error}");
                        continue;
                    }
                };

                if items.is_empty() {
                    println!("[API] No more data for '{endpoint}' at offset {offset}.");
                    received_empty_batch = true;
                    continue;
                }

                if sender.send(items).await.is_err() {
                    should_stop.store(true, Ordering::Relaxed);
                    break;
                }
            }

            if received_empty_batch {
                should_stop.store(true, Ordering::Relaxed);
            }
        }
    });

    let database_task =
        spawn_database_batch_writer(db_path, receiver, table_name, move |conn, items| {
            let transaction = conn.transaction()?;
            {
                let mut statement = transaction.prepare_cached(insert_sql)?;
                for item in items {
                    let params = params_fn(item);
                    let params: Vec<&dyn ToSql> = params
                        .iter()
                        .map(|value| value.as_ref() as &dyn ToSql)
                        .collect();
                    statement.execute(&params[..])?;
                }
            }
            transaction.commit()?;
            Ok(())
        });

    let (fetch_result, database_result) = tokio::join!(fetch_task, database_task);
    fetch_result?;
    database_result??;

    println!(
        "[SUCCESS] Synchronization for '{table_name}' completed in {:.2?}.",
        total_start.elapsed()
    );
    Ok(())
}

pub async fn run_id_sync_pipeline<T, F>(
    client_id: &str,
    access_token: &str,
    endpoint: &'static str,
    fields: &'static str,
    ids: Vec<i64>,
    db_path: PathBuf,
    item_label: &'static str,
    save_batch: F,
) -> Result<(), DynError>
where
    T: serde::de::DeserializeOwned + Send + 'static,
    F: Fn(&mut Connection, &[T]) -> Result<(), DynError> + Copy + Send + Sync + 'static,
{
    let total_start = Instant::now();
    let (sender, receiver) = mpsc::channel::<Vec<T>>(32);
    let client = build_igdb_client(client_id, access_token)?;

    let fetch_task = tokio::spawn(async move {
        let id_batches: Vec<Vec<i64>> = ids
            .chunks(API_BATCH_SIZE)
            .map(|batch| batch.to_vec())
            .collect();
        let mut workers = futures::stream::iter(id_batches)
            .map(|id_batch| {
                let client = client.clone();
                async move {
                    let requested_count = id_batch.len();
                    let query = format!(
                        "fields {fields}; where id = ({}); limit {API_BATCH_SIZE};",
                        id_batch
                            .iter()
                            .map(i64::to_string)
                            .collect::<Vec<_>>()
                            .join(",")
                    );
                    let request_start = Instant::now();
                    let response = client
                        .post(format!("https://api.igdb.com/v4/{endpoint}"))
                        .body(query)
                        .send()
                        .await?
                        .error_for_status()?;
                    let items = response.json::<Vec<T>>().await?;

                    Ok::<_, DynError>((requested_count, items, request_start.elapsed()))
                }
            })
            .buffer_unordered(REQUEST_CONCURRENCY);

        while let Some(result) = workers.next().await {
            let (requested_count, items, request_duration) = result?;
            println!(
                "[API] Found {} records for a batch of {} IDs in {:.2?}.",
                items.len(),
                requested_count,
                request_duration
            );

            if sender.send(items).await.is_err() {
                return Err("Database writer stopped before the fetch pipeline completed".into());
            }
        }

        Ok::<(), DynError>(())
    });

    let database_task = spawn_database_batch_writer(db_path, receiver, item_label, save_batch);
    let (fetch_result, database_result) = tokio::join!(fetch_task, database_task);
    fetch_result??;
    database_result??;

    println!(
        "[SUCCESS] Synchronization completed in {:.2?}.",
        total_start.elapsed()
    );
    Ok(())
}

fn build_igdb_client(client_id: &str, access_token: &str) -> Result<reqwest::Client, DynError> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "Client-ID",
        reqwest::header::HeaderValue::from_str(client_id)?,
    );
    headers.insert(
        "Authorization",
        reqwest::header::HeaderValue::from_str(&format!("Bearer {access_token}"))?,
    );

    Ok(reqwest::Client::builder()
        .default_headers(headers)
        .build()?)
}

fn spawn_database_batch_writer<T, F>(
    db_path: PathBuf,
    mut receiver: mpsc::Receiver<Vec<T>>,
    item_label: &'static str,
    save_batch: F,
) -> JoinHandle<Result<(), DynError>>
where
    T: Send + 'static,
    F: Fn(&mut Connection, &[T]) -> Result<(), DynError> + Copy + Send + Sync + 'static,
{
    tokio::task::spawn_blocking(move || {
        let mut conn = Connection::open(db_path)?;
        let mut batch = Vec::with_capacity(DATABASE_BATCH_SIZE);

        while let Some(items) = receiver.blocking_recv() {
            batch.extend(items);

            if batch.len() >= DATABASE_BATCH_SIZE {
                save_database_batch(&mut conn, &mut batch, item_label, save_batch)?;
            }
        }

        if !batch.is_empty() {
            save_database_batch(&mut conn, &mut batch, item_label, save_batch)?;
        }

        Ok(())
    })
}

fn save_database_batch<T, F>(
    conn: &mut Connection,
    batch: &mut Vec<T>,
    item_label: &str,
    save_batch: F,
) -> Result<(), DynError>
where
    F: Fn(&mut Connection, &[T]) -> Result<(), DynError>,
{
    let database_start = Instant::now();
    save_batch(conn, batch)?;
    println!(
        "[DB] Saved {} {} in {:.2?}.",
        batch.len(),
        item_label,
        database_start.elapsed()
    );
    batch.clear();

    Ok(())
}
