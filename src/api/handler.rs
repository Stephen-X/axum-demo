use crate::api::model::Value;
use axum::Router;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use tracing::info;
use crate::dependency::ApplicationState;

pub fn get_api_routes() -> Router<ApplicationState> {
    Router::new()
        .route("/{key}", get(read_by_key))
        .route("/{key}", post(upsert_by_key))
}

// Note: https://github.com/tokio-rs/axum/tree/main/examples/customize-extractor-error

/// Handler function to read a value by key from the database.
/// # Arguments
/// * `state`: The application state.
/// * `key`: The key to look up in the database.
async fn read_by_key(
    State(state): State<ApplicationState>,
    Path(key): Path<String>,
) -> Result<String, StatusCode> {
    let db = state.db.read().unwrap();

    if let Some(value) = db.read(&key) {
        Ok(value)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Handler function to upsert a value by key in the database.
/// # Arguments
/// * `state`: The application state.
/// * `key`: The key to upsert in the database.
/// * `payload`: The request payload that contains the value.
async fn upsert_by_key(
    State(state): State<ApplicationState>,
    Path(key): Path<String>,
    Json(payload): Json<Value>,
) -> Result<String, StatusCode> {
    let mut db = state.db.write().unwrap();

    if payload.value.is_empty() {
        info!("Value for key '{}' is empty, skipping upsert...", key);
        Err(StatusCode::BAD_REQUEST)
    } else {
        db.upsert(&key, payload.value);
        Ok(format!("Value written for key: {}", key))
    }
}
