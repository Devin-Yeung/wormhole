use crate::backend::WriteUrlCmd;
use crate::error::{AppError, Result};
use crate::model::{CreateUrlRequest, CreateUrlResponse, GetUrlResponse};
use crate::state::AppState;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use std::result::Result as StdResult;

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

pub async fn delete_url_handler(
    Path(short_code): Path<String>,
    State(state): State<AppState>,
) -> Result<StatusCode> {
    state.url_service().delete(&short_code).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::backend::{BackendError, GetUrlResult, UrlRead, UrlWrite, WriteUrlResult};
    use async_trait::async_trait;
    use axum::body::Body;
    use axum::http::{Method, Request};
    use tower::util::ServiceExt;

    #[derive(Clone)]
    struct FakeUrlService;

    #[async_trait]
    impl UrlWrite for FakeUrlService {
        async fn create(
            &self,
            _cmd: crate::backend::WriteUrlCmd,
        ) -> crate::backend::Result<WriteUrlResult> {
            Ok(WriteUrlResult {
                short_code: "abc123".to_string(),
                short_url: "https://worm.hole/abc123".to_string(),
                original_url: "https://example.com".to_string(),
                expire_at: None,
            })
        }

        async fn delete(&self, short_code: &str) -> crate::backend::Result<()> {
            if short_code == "missing" {
                return Err(BackendError::NotFound);
            }

            Ok(())
        }
    }

    #[async_trait]
    impl UrlRead for FakeUrlService {
        async fn get(&self, short_code: &str) -> crate::backend::Result<GetUrlResult> {
            if short_code == "missing" {
                return Err(BackendError::NotFound);
            }

            Ok(GetUrlResult {
                original_url: "https://example.com".to_string(),
                expire_at: None,
            })
        }
    }

    fn test_app() -> axum::Router {
        let state = AppState::builder()
            .url_service(FakeUrlService)
            .base_url("https://worm.hole".to_string())
            .build();
        App::router(state)
    }

    #[tokio::test]
    async fn create_handler_returns_created() {
        let app = test_app();
        let request = Request::builder()
            .method(Method::POST)
            .uri("/v1/urls")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "original_url": "https://example.com",
                    "custom_alias": null,
                    "expire_at": null
                })
                .to_string(),
            ))
            .expect("request should build");

        let response = app.oneshot(request).await.expect("request should succeed");

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn get_handler_returns_not_found() {
        let app = test_app();
        let request = Request::builder()
            .method(Method::GET)
            .uri("/v1/urls/missing")
            .body(Body::empty())
            .expect("request should build");

        let response = app.oneshot(request).await.expect("request should succeed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_handler_returns_no_content() {
        let app = test_app();
        let request = Request::builder()
            .method(Method::DELETE)
            .uri("/v1/urls/abc123")
            .body(Body::empty())
            .expect("request should build");

        let response = app.oneshot(request).await.expect("request should succeed");

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }
}
