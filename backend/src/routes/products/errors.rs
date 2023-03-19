use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use serde_json::json;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum ProductError {
    #[error("Rating already exists")]
    AlreadyRated,
    #[error("Unknown product")]
    UnknownProduct,
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl IntoResponse for ProductError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match &self {
            ProductError::AlreadyRated => StatusCode::BAD_REQUEST,
            ProductError::UnknownProduct => StatusCode::BAD_REQUEST,
            ProductError::Unexpected(e) => {
                error!("Internal server error: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        let info = match self {
            ProductError::Unexpected(_) => "Unexpected server error".to_string(),
            _ => self.to_string(),
        };

        (status_code, Json(json!({ "errorInfo": info }))).into_response()
    }
}

impl From<sqlx::Error> for ProductError {
    fn from(e: sqlx::Error) -> Self {
        Self::Unexpected(anyhow::Error::from(e))
    }
}