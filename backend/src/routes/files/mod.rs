use axum::{extract::Multipart, routing::post, Router, Json};
use axum::extract::Path;
use axum::http::{StatusCode, Uri};
use axum::response::IntoResponse;
use axum::routing::get;
use base64::Engine;
use tracing::debug;
use tracing::field::debug;
use uuid::Uuid;
use crate::AppState;


async fn fetch(Path(file_id): Path<Uuid>) -> Result<Vec<u8>, StatusCode>{
    let res = tokio::fs::read(format!("../files/{file_id}")).await;
    if let Ok(buf) = res {
        let base = base64::engine::general_purpose::STANDARD;

        return Ok(
            base.decode(buf).unwrap()
        );

    }
    Err(StatusCode::BAD_REQUEST)

}

pub fn router() -> Router<AppState> {
    Router::new().route("/fetch/:file_id", get(fetch))
}