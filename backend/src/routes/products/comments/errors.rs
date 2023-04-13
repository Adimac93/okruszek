use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use serde_json::json;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum CommentError {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl IntoResponse for CommentError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match &self {
            CommentError::Unexpected(e) => {
                error!("Internal server error: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        let info = match self {
            CommentError::Unexpected(_) => "Unexpected server error".to_string(),
            _ => self.to_string(),
        };

        (status_code, Json(json!({ "errorInfo": info }))).into_response()
    }
}

impl From<sqlx::Error> for CommentError {
    fn from(e: sqlx::Error) -> Self {
        Self::Unexpected(anyhow::Error::from(e))
    }
}