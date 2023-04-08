mod errors;

use std::collections::HashMap;
use crate::routes::products::errors::ProductError;
use crate::AppState;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{debug_handler, Json, Router};
use axum::body::Bytes;
use axum::http::header::CONTENT_TYPE;
use base64::Engine;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use sqlx::{query, query_as, PgPool};
use tracing::{debug, error};
use typeshare::typeshare;
use crate::routes::auth::session::Claims;
use crate::routes::files::BucketClient;

pub fn router() -> Router<AppState> {

    Router::new()
        .route("/", get(fetch_all).put(add))
        .route("/ratings/:product_id", get(ratings).put(rate))


}

#[typeshare]
#[derive(Serialize)]
struct Product {
    name: String,
    price: f64,
    rating: Option<i32>,
    image: Option<String>,
}

async fn fetch_all(session: Claims, pool: State<PgPool>, State(BucketClient(client)): State<BucketClient>) -> Result<Json<HashMap<Uuid,Product>>, ProductError> {
    let mut conn = pool.acquire().await.unwrap();
    let res = query!(r#"
    SELECT id, name, price, rating
    FROM products p
    LEFT JOIN product_ratings pr ON pr.product_id = p.id AND pr.user_id = $1
    "#, session.user_id
    )
    .fetch_all(&mut *conn)
    .await?;


    let mut products: HashMap<Uuid, Product> = res.iter().map(|product| (product.id, Product{
        name: product.name.clone(),
        price: product.price,
        rating: product.rating,
        image: None
    } )).collect();

    let product_ids = res.iter().map(|x| x.id);
    for product_id in product_ids {
        let res = client
            .get(format!("http://127.0.0.1:3001/download/{product_id}"))
            .send().await.ok();

        if let Some(res) = res {
            if let Some(product) = products.get_mut(&product_id) {
                let encoded = base64::engine::general_purpose::STANDARD.encode(res.bytes().await.unwrap());
                product.image = Some(encoded);
            }
        }


    }

    Ok(Json(products))
}

#[typeshare]
#[derive(Serialize)]
struct Rating {
    username: String,
    rating: i32,
}

#[debug_handler]
async fn ratings(session: Claims, pool: State<PgPool>, Path(product_id): Path<Uuid>) -> Result<Json<Vec<Rating>>, ProductError> {
    let mut conn = pool.acquire().await?;
    let ratings = query_as!(Rating, r#"
    SELECT username, rating
    FROM product_ratings pr
    JOIN users ON users.id = pr.user_id
    WHERE pr.product_id = $1 AND pr.user_id <> $2
    "#, product_id, session.user_id
    ).fetch_all(&mut *conn).await?;

    Ok(Json(ratings))
}

#[typeshare]
#[derive(Deserialize)]
struct Rate {
    rating: i32,
}

async fn rate(session: Claims, pool: State<PgPool>, Path(product_id): Path<Uuid>, Json(body): Json<Rate>) -> Result<(), ProductError> {
    let mut transaction = pool.begin().await?;
    let res = query!(r#"
    SELECT *
    FROM product_ratings
    WHERE product_id = $1 AND user_id = $2
    "#, product_id, session.user_id).fetch_optional(&mut *transaction).await?;

    if res.is_some() {
        return Err(ProductError::AlreadyRated)
    }

    query!(r#"
    INSERT INTO product_ratings (product_id, user_id, rating)
    VALUES ($1, $2, $3)
    "#, product_id, session.user_id, body.rating).execute(&mut *transaction).await?;

    transaction.commit().await?;
    Ok(())
}


#[typeshare]
#[derive(Deserialize)]
struct AddProduct {
    name: String,
    price: f64,
    image: Option<String>,
}

async fn add(session: Claims, State(pool): State<PgPool>, State(BucketClient(client)): State<BucketClient>, Json(body): Json<AddProduct>) -> Result<(), ProductError> {
    if let Some(image) = body.image {
        let decoded = base64::engine::general_purpose::STANDARD.decode(image).unwrap();
        let bytes = Bytes::from(decoded);
        let part = reqwest::multipart::Part::bytes(bytes.to_vec()).file_name("name.png");
        let form = reqwest::multipart::Form::new().part("file", part);
        let res = client
            .post("http://127.0.0.1:3001/upload")
            .multipart(form)
            .header(CONTENT_TYPE, "multipart/form-data")
            .send().await.unwrap();

        let json = res.json::<Vec<Uuid>>().await.unwrap();
        let file_id = json[0];

        let _id = query!(r#"
        INSERT INTO products (id, name, price)
        VALUES ($1, $2, $3)
        "#, file_id, body.name, body.price).execute(&pool).await?;
    }

    Ok(())
}