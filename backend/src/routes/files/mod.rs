use std::io::Read;
use axum::{routing::post, Router, Json, Form, debug_handler};
use axum::body::{Bytes, StreamBody};
use axum::extract::{Multipart, Path, State};
use axum::http::{StatusCode, Uri};
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use base64::Engine;
use hyper::Body;
use hyper::header::CONTENT_TYPE;
use reqwest::{Client, RequestBuilder, Response};
use reqwest::header::{AUTHORIZATION, HeaderMap};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use tracing::debug;
use tracing_test::traced_test;
use uuid::Uuid;
use crate::AppState;
use crate::routes::auth::session::Claims;


pub fn router() -> Router<AppState> {
    Router::new()
        .route("/upload", post(upload))
        .route("/download/:file_id", get(download))
}

#[derive(Clone)]
pub struct BucketClient(pub Client);

impl BucketClient {
    pub async fn new() -> Self {
        let auth_key = if let Ok(file_content) = tokio::fs::read_to_string("./key.json").await {
            debug!("Reading auth key from file");
            serde_json::from_str::<AuthKey>(&file_content).unwrap()
        } else {
            debug!("Issuing new key from API");
            let res = Client::new()
                .get("http://127.0.0.1:3001/key")
                .send().await.unwrap();

            if !res.status().is_success() {
                panic!("Failed to issue new bucket key");
            }

            let key = res.json::<AuthKey>().await.unwrap();
            let payload = serde_json::to_string_pretty(&key).unwrap();
            debug!("Saving key to file");
            tokio::fs::write("./key.json", payload).await.unwrap();
            key
        };
        let id = auth_key.key_id;
        let key = auth_key.key;

        let mut headers = HeaderMap::new();
        let encoded_key = base64::engine::general_purpose::STANDARD.encode(format!("{id}:{key}"));
        headers.append(AUTHORIZATION, format!("Basic {encoded_key}").parse().unwrap());
        let client = Client::builder().default_headers(headers).build().unwrap();

        Self(client)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all="camelCase")]
struct AuthKey {
    key_id: Uuid,
    key: Uuid,
}

#[debug_handler]
async fn upload(State(BucketClient(client)): State<BucketClient>, mut multipart: Multipart) -> impl IntoResponse {
    let mut form = reqwest::multipart::Form::new();
    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = field.file_name().unwrap().to_string();
        let bytes = field.bytes().await.unwrap().to_vec();
        let part = reqwest::multipart::Part::bytes(bytes).file_name(file_name);
        form = form.part("file", part);
    }

    debug!("Uploading files to bucket service");
    let res = client
        .post("http://127.0.0.1:3001/upload")
        .multipart(form)
        .header(CONTENT_TYPE, "multipart/form-data")
        .send().await.unwrap();

    debug!("{}", res.status());
    let json = res.json::<Value>().await.unwrap();
    Json(json)
}

#[debug_handler]
async fn download(State(BucketClient(client)): State<BucketClient>, Path(file_id): Path<Uuid>) -> impl IntoResponse {
    let res = client
        .get(format!("http://127.0.0.1:3001/download/{file_id}"))
        .send().await.unwrap();


    let body = StreamBody::from(res.bytes_stream());
    body
}

async fn key() {
    let res = Client::new()
        .get("http://127.0.0.1:3001/key")
        .send().await.unwrap();

    let json = res.json::<AuthKey>().await.unwrap();
    println!("{json:#?}");


    let file = tokio::fs::read("./store/file.png").await.unwrap();
    let file_part = reqwest::multipart::Part::bytes(file).file_name("file.png").mime_str("image/png").unwrap();
    let form = reqwest::multipart::Form::new().part("file", file_part);
    let res = Client::new()
        .post("http://127.0.0.1:3001/upload")
        .basic_auth(json.key_id, Some(json.key))
        .multipart(form)
        .send().await.unwrap();

    let file_ids = res.json::<Vec<Uuid>>().await.unwrap();
    let file_id = file_ids[0];
    println!("{file_id}");
    let res = Client::new()
        .get(format!("http://127.0.0.1:3001/download/{file_id}"))
        .basic_auth(json.key_id, Some(json.key))
        .send().await.unwrap();

    let res = Client::new()
        .get(format!("http://127.0.0.1:3001/delete/{file_id}"))
        .basic_auth(json.key_id, Some(json.key))
        .send().await.unwrap();
}

#[tokio::test]
async fn a() {
    key().await;
}

#[traced_test]
#[tokio::test]
async fn b() {

}