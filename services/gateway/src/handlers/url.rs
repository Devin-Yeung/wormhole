use crate::backend::{DeleteUrlCmd, WriteUrlCmd};
use crate::error::{AppError, Result};
use crate::model::{CreateUrlRequest, CreateUrlResponse, GetUrlResponse};
use crate::state::AppState;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use std::result::Result as StdResult;
use tracing::instrument;

#[instrument(skip(state))]
pub async fn create_url_handler(
    State(state): State<AppState>,
    request: StdResult<Json<CreateUrlRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<CreateUrlResponse>)> {
    let Json(request) =
        request.map_err(|rejection| AppError::invalid_request(rejection.body_text()))?;

    let result = state
        .url_service()
        .create(WriteUrlCmd {
            original_url: request.original_url,
            custom_alias: request.custom_alias,
            expire_at: request.expire_at,
        })
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateUrlResponse {
            short_code: result.short_code,
            short_url: result.short_url,
            original_url: result.original_url,
            expire_at: result.expire_at,
        }),
    ))
}

#[instrument(skip(state))]
pub async fn get_url_handler(
    Path(short_code): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<GetUrlResponse>> {
    let result = state.url_service().get(&short_code).await?;

    Ok(Json(GetUrlResponse {
        original_url: result.original_url,
        expire_at: result.expire_at,
    }))
}

#[instrument(skip(state))]
pub async fn delete_url_handler(
    Path(short_code): Path<String>,
    State(state): State<AppState>,
) -> Result<StatusCode> {
    state
        .url_service()
        .delete(DeleteUrlCmd { short_code })
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
