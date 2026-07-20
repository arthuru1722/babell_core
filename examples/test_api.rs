use axum::{
    extract::{Query},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use std::fs;

// Importação do middleware de CORS
use tower_http::cors::{Any, CorsLayer};

// Importações do seu projeto (babel_core)
use babel_core::igdb::database::models::SearchGame;
use babel_core::igdb::pipeline::DynError;
use babel_core::utils::paths::get_base_directory;

// CORREÇÃO: Importando o SourceData diretamente do módulo flaresolverr correto
use babel_core::flaresolverr::SourceData; 

use rusqlite::{params, Connection};
use tokio::task;

// --- Estruturas de Parâmetros da URL ---

#[derive(Deserialize)]
struct PaginationParams {
    limit: Option<usize>,
    offset: Option<usize>,
}

#[derive(Deserialize)]
struct SearchParams {
    q: String,
    limit: Option<usize>,
    offset: Option<usize>,
}

#[derive(Deserialize)]
struct SourceSearchParams {
    q: String,
}

// --- Handlers do Axum ---

/// Handler para GET /games
async fn get_games_handler(
    Query(params): Query<PaginationParams>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);

    match list_games(limit, offset).await {
        Ok(games) => (StatusCode::OK, Json(games)).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erro no banco de dados: {err}"),
        )
            .into_response(),
    }
}

/// Handler para GET /games/search
async fn search_games_handler(
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);

    if params.q.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            "O parâmetro de busca 'q' não pode estar vazio.",
        )
            .into_response();
    }

    match search_games(&params.q, limit, offset).await {
        Ok(games) => (StatusCode::OK, Json(games)).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erro no banco de dados: {err}"),
        )
            .into_response(),
    }
}

/// Handler para GET /sources - Lista apenas os nomes das fontes salvas
async fn list_sources_handler() -> impl IntoResponse {
    match task::spawn_blocking(list_source_names).await {
        Ok(Ok(names)) => (StatusCode::OK, Json(names)).into_response(),
        Ok(Err(err)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erro ao ler nomes das fontes: {err}"),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erro de concorrência: {err}"),
        )
            .into_response(),
    }
}

/// Handler para GET /sources/search - Pesquisa por um termo no campo 'title' das fontes
async fn search_sources_handler(
    Query(params): Query<SourceSearchParams>,
) -> impl IntoResponse {
    if params.q.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            "O parâmetro de busca 'q' não pode estar vazio.",
        )
            .into_response();
    }

    match task::spawn_blocking(move || search_in_sources(&params.q)).await {
        Ok(Ok(filtered_sources)) => (StatusCode::OK, Json(filtered_sources)).into_response(),
        Ok(Err(err)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erro ao pesquisar nas fontes: {err}"),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erro de concorrência: {err}"),
        )
            .into_response(),
    }
}

// --- Função Principal ---

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/games", get(get_games_handler))
        .route("/games/search", get(search_games_handler))
        .route("/sources", get(list_sources_handler))
        .route("/sources/search", get(search_sources_handler))
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Servidor Axum rodando em http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// --- Funções Auxiliares para ler os JSONs ---

fn list_source_names() -> Result<Vec<String>, DynError> {
    let mut path = get_base_directory().map_err(|e| e.to_string())?;
    path.push("sources");

    if !path.exists() {
        return Ok(vec![]);
    }

    let mut names = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();

        if entry_path.is_dir() {
            if let Some(folder_name) = entry_path.file_name().and_then(|n| n.to_str()) {
                names.push(folder_name.to_string());
            }
        }
    }

    Ok(names)
}

fn load_all_sources() -> Result<Vec<SourceData>, DynError> {
    let mut path = get_base_directory().map_err(|e| e.to_string())?;
    path.push("sources");

    if !path.exists() {
        return Ok(vec![]);
    }

    let mut all_sources = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();

        if entry_path.is_dir() {
            for file_entry in fs::read_dir(entry_path)? {
                let file_entry = file_entry?;
                let file_path = file_entry.path();

                if file_path.extension().and_then(|s| s.to_str()) == Some("json") {
                    let content = fs::read_to_string(&file_path)?;
                    if let Ok(source_data) = serde_json::from_str::<SourceData>(&content) {
                        all_sources.push(source_data);
                    }
                }
            }
        }
    }

    Ok(all_sources)
}

fn search_in_sources(query: &str) -> Result<Vec<SourceData>, DynError> {
    let all_sources = load_all_sources()?;
    let query_lowercase = query.to_lowercase();
    let mut filtered_sources = Vec::new();

    for source in all_sources {
        let matching_downloads: Vec<_> = source.downloads
            .into_iter()
            .filter(|d| d.title.to_lowercase().contains(&query_lowercase))
            .collect();

        if !matching_downloads.is_empty() {
            filtered_sources.push(SourceData {
                name: source.name,
                downloads: matching_downloads,
            });
        }
    }

    Ok(filtered_sources)
}

// --- Funções de Banco de Dados (Originais) ---

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
                game_id, name, cover_url, release_date, rating,
                aggregated_rating, summary, storyline, updated_at
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
                game_id, name, cover_url, release_date, rating,
                aggregated_rating, summary, storyline, updated_at
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