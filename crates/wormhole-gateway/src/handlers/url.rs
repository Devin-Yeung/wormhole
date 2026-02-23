use crate::error::Result;
use crate::model::{CreateUrlRequest, CreateUrlResponse, GetUrlResponse};
use crate::state::AppState;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use std::result::Result as StdResult;

pub async fn create_url_handler(
    State(_state): State<AppState>,
    _request: StdResult<Json<CreateUrlRequest>, JsonRejection>,
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
) -> Result<StatusCode> {
    todo!()
}
