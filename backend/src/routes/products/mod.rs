mod errors;

use std::collections::HashMap;
use crate::routes::products::errors::ProductError;
use crate::AppState;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{debug_handler, Json, Router};
use axum::body::Bytes;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use sqlx::{query, query_as, PgPool};
use tracing::{debug, error};
use crate::routes::auth::session::Session;
use crate::routes::files::get_file_path;

pub fn router() -> Router<AppState> {

    Router::new()
        .route("/", get(fetch_all).put(add))
        .route("/ratings/:product_id", get(ratings).put(rate))


}

#[derive(Serialize)]
struct Product {
    name: String,
    price: f64,
    rating: Option<i32>,
    image: Option<String>,
}

async fn fetch_all(session: Session, pool: State<PgPool>) -> Result<Json<HashMap<Uuid,Product>>, ProductError> {
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
        let res = tokio::fs::read_to_string(get_file_path(product_id)).await;
        if let Ok(buf) = res {
            if let Some(product) = products.get_mut(&product_id) {
                product.image = Some(buf);
            }
        }
    }

    Ok(Json(products))
}

#[derive(Serialize)]
struct Rating {
    username: String,
    rating: i32,
}

#[debug_handler]
async fn ratings(session: Session, pool: State<PgPool>, Path(product_id): Path<Uuid>) -> Result<Json<Vec<Rating>>, ProductError> {
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

#[derive(Deserialize)]
struct Rate {
    rating: i32,
}

async fn rate(session: Session, pool: State<PgPool>, Path(product_id): Path<Uuid>, Json(body): Json<Rate>) -> Result<(), ProductError> {
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


#[derive(Deserialize)]
struct AddProduct {
    name: String,
    price: f64,
    image: Option<String>,
}

async fn add(session: Session, pool: State<PgPool>, Json(body): Json<AddProduct>) -> Result<(), ProductError> {
    let mut conn = pool.acquire().await?;
    let id = query!(r#"
    INSERT INTO products (name, price)
    VALUES ($1, $2)
    RETURNING id
    "#, body.name, body.price).fetch_one(&mut *conn).await?.id;

    if let Some(image) = body.image {
        let bytes = Bytes::from(image);
        match tokio::fs::write(get_file_path(id),bytes).await {
            Ok(_) => debug!("Saved product image"),
            Err(e) => error!("Failed to save a file {e}")
        }
    }

    Ok(())
}