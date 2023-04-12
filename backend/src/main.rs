use std::{env, fs};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;
use dotenv::dotenv;
use sha1::{Sha1, Digest};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tracing::{debug, error, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use backend::{app, AppState, Environment};

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();




    let env = env::var("APP_ENVIRONMENT").expect("APP_ENVIRONMENT var missing");
    let environment = Environment::try_from(env).unwrap();


    let addr = match environment {
        Environment::Development => {

            let output_dir = "../frontend/src/lib/interfaces.ts";

            let prev_checksum = get_file_checksum(output_dir).await;


            // depends on typeshare-cli
            let _output = Command::new("typeshare")
                .arg(".")
                .arg("--lang=typescript")
                .arg("--output-file=../frontend/src/lib/interfaces.ts")
                .output()
                .await
                .expect("Failed to execute typeshare `cargo install typeshare-cli`");

            let curr_checksum = get_file_checksum(output_dir).await;

            match (prev_checksum, curr_checksum) {
                (Some(p), Some(c)) => {
                    if p == c {
                        info!("Typeshare interfaces unchanged");
                    } else {
                        info!("Typeshare interfaces updated");
                    }
                },
                (None, Some(_)) => {
                    info!("Typeshare interfaces created");
                }
                _ => {
                    error!("Typesahre interfaces error");
                }
            }

            SocketAddr::from(([127, 0, 0, 1], 3000))
        },
        Environment::Production => {
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

            let port = env::var("PORT").expect("PORT var missing").parse::<u16>().expect("Failed to parse PORT var");
            SocketAddr::from(([0, 0, 0, 0], port)) }
    };

    let app_state = AppState::new(environment).await;
    info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(
            app(app_state).into_make_service()
        ).await
        .expect("Failed to run axum server");
}

async fn get_file_checksum(path: impl AsRef<Path>) -> Option<String> {
    if let Ok(mut file) = File::open(path).await {
        let mut buf = String::new();
        if let Ok(size) = file.read_to_string(&mut buf).await {
            let mut hasher = Sha1::new();
            hasher.update(buf.as_bytes());
            let hash = hasher.finalize();
            let checksum = format!("{hash:x}");
            return Some(checksum);
        }
    }
    None
}