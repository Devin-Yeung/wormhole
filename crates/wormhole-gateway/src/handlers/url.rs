use crate::error::Result;
use crate::model::{CreateUrlRequest, CreateUrlResponse, DeleteUrlResponse, GetUrlResponse};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;

pub async fn create_url_handler(
    State(_state): State<AppState>,
    Json(_request): Json<CreateUrlRequest>,
) -> Result<Json<CreateUrlResponse>> {
    todo!()
}

pub async fn get_url_handler(
    Path(_short_code): Path<String>,
    State(_state): State<AppState>,
) -> Result<Json<GetUrlResponse>> {
    todo!()
}

pub async fn delete_url_handler(
    Path(_short_code): Path<String>,
    State(_state): State<AppState>,
) -> Result<Json<DeleteUrlResponse>> {
    todo!()
}
