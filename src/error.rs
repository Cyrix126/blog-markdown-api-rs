use axum::http::StatusCode;
use axum_thiserror::ErrorStatus;
use thiserror::Error;

#[derive(Debug, Error, ErrorStatus)]
pub enum AppError {
    #[error("post not found")]
    #[status(StatusCode::NOT_FOUND)]
    NotFound,
    #[error("could not write file")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    IoError(#[from] std::io::Error),
}
