use crate::model::HealthResponse;
use axum::Json;

pub async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}
