//! HTTP API handlers for the block explorer

use super::service::ExplorerService;
use super::types::*;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

/// Application state shared across handlers
pub type AppState = Arc<ExplorerService>;

/// Create the API router
pub fn create_router(explorer: Arc<ExplorerService>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Static files / UI
        .route("/", get(index_handler))
        // API routes
        .route("/api/v1/stats", get(get_stats))
        .route("/api/v1/blocks", get(list_blocks))
        .route("/api/v1/blocks/:height", get(get_block))
        .route("/api/v1/tasks", get(list_tasks))
        .route("/api/v1/search", get(search))
        .layer(cors)
        .with_state(explorer)
}

/// Serve the main HTML page
async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../../explorer-ui/index.html"))
}

/// Get chain statistics
async fn get_stats(State(explorer): State<AppState>) -> Json<ChainStats> {
    Json(explorer.get_stats())
}

/// Query parameters for listing blocks
#[derive(Debug, serde::Deserialize)]
pub struct ListBlocksParams {
    limit: Option<usize>,
}

/// List recent blocks
async fn list_blocks(
    State(explorer): State<AppState>,
    Query(params): Query<ListBlocksParams>,
) -> Json<Vec<BlockSummary>> {
    let limit = params.limit.unwrap_or(10).min(100);
    Json(explorer.get_recent_blocks(limit))
}

/// Get a specific block by height
async fn get_block(
    State(explorer): State<AppState>,
    Path(height): Path<u64>,
) -> Response {
    match explorer.get_block(height) {
        Some(block) => Json(block).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiError::not_found(&format!("Block {} not found", height))),
        )
            .into_response(),
    }
}

/// Query parameters for listing tasks
#[derive(Debug, serde::Deserialize)]
pub struct ListTasksParams {
    status: Option<String>,
}

/// List tasks with optional status filter
async fn list_tasks(
    State(explorer): State<AppState>,
    Query(params): Query<ListTasksParams>,
) -> Json<Vec<TaskView>> {
    Json(explorer.get_tasks(params.status))
}

/// Query parameters for search
#[derive(Debug, serde::Deserialize)]
pub struct SearchParams {
    q: String,
}

/// Universal search
async fn search(
    State(explorer): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Json<SearchResults> {
    Json(explorer.search(&params.q))
}
