use axum::response::{IntoResponse, Response};

pub type Result<T> = std::result::Result<T, AppError>;

pub enum AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        todo!()
    }
}
