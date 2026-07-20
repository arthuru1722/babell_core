use axum::{
    extract::Query,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::net::SocketAddr;
use tokio::net::TcpListener;

// Importação do middleware de CORS
use tower_http::cors::{Any, CorsLayer};

// Importações do seu projeto (babel_core)
use babel_core::igdb::database::models::SearchGame;
use babel_core::igdb::pipeline::DynError;
use babel_core::utils::paths::get_base_directory;

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

// --- Função Principal ---

#[tokio::main]
async fn main() {
    // Configuração do CORS para permitir que o arquivo HTML (index.html) faça requisições
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Define as rotas e aplica a camada de CORS globalmente
    let app = Router::new()
        .route("/games", get(get_games_handler))
        .route("/games/search", get(search_games_handler))
        .layer(cors); // <-- Aplica o middleware aqui

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Servidor Axum rodando em http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// --- Funções de Banco de Dados ---

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