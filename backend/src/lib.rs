use std::env;
use axum::extract::FromRef;
use axum::{debug_handler, Router};
use axum::http::{StatusCode, Uri};
use axum::response::IntoResponse;
use sqlx::{migrate, PgPool};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use crate::routes::{auth, files, products};
use crate::routes::files::BucketClient;

pub mod routes;


pub fn app(app_state: AppState) -> Router {

    let api = Router::new()
        .nest("/auth", auth::router())
        .nest("/products",products::router())
        .nest("/files", files::router());

    Router::new()
        .nest("/api", api)
        .nest_service(
        "/",
        ServeDir::new("../frontend/dist").not_found_service(ServeFile::new("../frontend/dist/index.html")),
        )
        .fallback(not_found)
        .with_state(app_state)


}

async fn not_found(
) -> impl IntoResponse {
  (StatusCode::NOT_FOUND, "404 Not Found")
}


#[derive(FromRef, Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub bucket_client: BucketClient,
}

impl AppState {
    pub async fn new(environment: Environment) -> Self {
        let pool = PgPool::connect(&env::var("DATABASE_URL").expect("DATABASE_URL var missing")).await.unwrap();
        if environment == Environment::Production {
            migrate!("./migrations").run(&pool).await.expect("Failed to migrate");
        }
        let bucket_client = BucketClient::new().await;
        Self { pool, bucket_client }
    }
}

#[derive(PartialEq)]
pub enum Environment {
    Development,
    Production
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "development" | "dev" => Ok(Self::Development),
            "production" | "prod" => Ok(Self::Production),
            other => Err(format!(
                "{other} is not supported environment. Use either `local` or `production`"
            )),
        }
    }
}
