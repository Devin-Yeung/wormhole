use crate::error::{AppError, Result};
use crate::model::{CreateUrlRequest, CreateUrlResponse, DeleteUrlResponse, GetUrlResponse};
use crate::state::AppState;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use jiff::Timestamp;
use std::result::Result as StdResult;
use wormhole_core::ShortCode;
use wormhole_shortener::shortener::{ExpirationPolicy, ShortenParams};

pub async fn create_url_handler(
    State(state): State<AppState>,
    request: StdResult<Json<CreateUrlRequest>, JsonRejection>,
) -> Result<impl IntoResponse> {
    let Json(request) =
        request.map_err(|rejection| AppError::invalid_request(rejection.body_text()))?;
    let CreateUrlRequest {
        original_url,
        custom_alias,
        expire_at,
    } = request;

    let expiration = match expire_at.as_deref() {
        Some(raw_expire_at) => {
            // v1 REST requires explicit UTC timestamps to keep API behavior deterministic.
            if !raw_expire_at.ends_with('Z') {
                return Err(AppError::invalid_request(
                    "expire_at must be an RFC3339 UTC timestamp ending with 'Z'",
                ));
            }

            let timestamp = raw_expire_at.parse::<Timestamp>().map_err(|_| {
                AppError::invalid_request(
                    "expire_at must be an RFC3339 UTC timestamp ending with 'Z'",
                )
            })?;

            ExpirationPolicy::AtTimestamp(timestamp)
        }
        None => ExpirationPolicy::Never,
    };

    let custom_alias = custom_alias
        .map(ShortCode::custom)
        .transpose()
        .map_err(|error| AppError::InvalidShortCode(error.to_string()))?;

    let short_code = state
        .shortener()
        .shorten(ShortenParams {
            original_url: original_url.clone(),
            expiration,
            custom_alias,
        })
        .await
        .map_err(AppError::from)?;

    let location = format!("/v1/urls/{short_code}");
    let response = CreateUrlResponse {
        short_code: short_code.to_string(),
        short_url: short_code.to_url(state.base_url()),
        original_url,
        expire_at,
    };

    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, location)],
        Json(response),
    ))
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
