use crate::error::Result;
use crate::model::{CreateUrlRequest, HealthResponse, UrlResponse};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;

pub async fn create_url_handler(
    State(state): State<AppState>,
    Json(request): Json<CreateUrlRequest>,
) -> Result<Response> {
    todo!()
}

pub async fn get_url_handler(
    Path(short_code): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<UrlResponse>> {
    todo!()
}

pub async fn delete_url_handler(
    Path(short_code): Path<String>,
    State(state): State<AppState>,
) -> Result<StatusCode> {
    todo!()
}
