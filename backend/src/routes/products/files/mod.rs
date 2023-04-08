use axum::{Router, Json, debug_handler};
use axum::body::{Bytes};
use axum::extract::{FromRef, Multipart, Path, State};
use base64::Engine;
use hyper::header::CONTENT_TYPE;
use reqwest::Client;
use reqwest::header::{AUTHORIZATION, HeaderMap};
use serde::{Serialize, Deserialize};
use tokio::task::{JoinError, JoinSet};
use tracing::debug;
use typeshare::typeshare;
use uuid::Uuid;

#[typeshare]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all="camelCase")]
struct AuthKey {
    key_id: Uuid,
    key: Uuid,
}


#[derive(Clone, FromRef)]
pub struct BucketClient {
    client: Client,
    url: String,
}

impl BucketClient {
    pub async fn new(url: &str) -> Self {
        let auth_key = if let Ok(file_content) = tokio::fs::read_to_string("./key.json").await {
            debug!("Reading auth key from file");
            serde_json::from_str::<AuthKey>(&file_content).unwrap()
        } else {
            debug!("Issuing new key from API");
            let res = Client::new()
                .get(format!("{}/key", url))
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

        Self {client, url: url.to_string()}
    }

    pub async fn upload(&self, image: String) -> anyhow::Result<Uuid> {
        let decoded = base64::engine::general_purpose::STANDARD.decode(image)?;
        let bytes = Bytes::from(decoded);
        let part = reqwest::multipart::Part::bytes(bytes.to_vec()).file_name("name.png");
        let form = reqwest::multipart::Form::new().part("file", part);
        let res = self.client
            .post(format!("{}/upload", self.url))
            .multipart(form)
            .header(CONTENT_TYPE, "multipart/form-data")
            .send().await?;

        let json = res.json::<Vec<Uuid>>().await?;
        let file_id = json[0];
        Ok(file_id)
    }

    pub async fn download(&self, file_id: Uuid) -> anyhow::Result<(Uuid,String)> {
        let res = self.client
            .get(format!("{}/download/{file_id}", self.url))
            .send().await?;

        let encoded = base64::engine::general_purpose::STANDARD.encode(res.bytes().await?);
        return Ok((file_id, encoded));
    }

    pub async fn download_many(&self, files_ids: Vec<(Uuid, Uuid)>) -> JoinSet<(Uuid, Result<(Uuid, String), anyhow::Error>)> {
        let mut set = JoinSet::new();
        //let mut images: Vec<(Uuid, String)> = Vec::with_capacity(files_ids.len());

        for (product_id, file_id) in files_ids {
            let cloned_client = self.clone();
            set.spawn(async move {
                (product_id, cloned_client.download(file_id).await)
            });
        }


        set
    }
}