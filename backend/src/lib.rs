use std::env;
use axum::extract::FromRef;
use axum::Router;
use axum::http::{StatusCode, Uri};
use axum::response::IntoResponse;
use sqlx::{migrate, PgPool};
use tokio::process::Command;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tracing::debug;
use uuid::Uuid;
use crate::routes::{auth, products};
use crate::routes::products::files::{AuthKey, BucketClient};

pub mod routes;


pub fn app(app_state: AppState) -> Router {

    let api = Router::new()
        .nest("/auth", auth::router())
        .nest("/products",products::router());

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
            debug!("Migrating database");
            migrate!("./migrations").run(&pool).await.expect("Failed to migrate");

            debug!("Reading bucket storage environment variables");
            let key = env::var("BUCKET_KEY").expect("BUCKET_KEY var missing");
            let id = env::var("BUCKET_KEY_ID").expect("BUCKET_KEY_ID var missing");
            let bucket_url = env::var("BUCKET_URL").expect("BUCKET_URL var missing");

            let bucket_client = match (key.parse::<Uuid>(), id.parse::<Uuid>()) {
                (Ok(key), Ok(id)) => {
                    debug!("Connecting with bucket storage at `{bucket_url}`");
                    BucketClient::from_key(&bucket_url, AuthKey::new(key, id)).await
                }
                _ => panic!("Failed to parse key part")
            };
            return Self {pool, bucket_client}
        }

        // blocking
        debug!("Compiling frontend");
        let status = Command::new("npm")
            .arg("run")
            .arg("build")
            .current_dir("../frontend")
            .spawn().unwrap()
            .wait().await.unwrap();

        if status.success() {
            debug!("Successfully compiled frontend");
        }

        let bucket_client = BucketClient::new("http://127.0.0.1:3001").await;
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
