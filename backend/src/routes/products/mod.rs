mod errors;
pub mod files;

use std::collections::HashMap;
use crate::routes::products::errors::ProductError;
use crate::AppState;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{debug_handler, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use sqlx::{query, query_as, PgPool};
use tracing::{debug, error};
use typeshare::typeshare;
use crate::routes::auth::session::Claims;

pub fn router() -> Router<AppState> {

    Router::new()
        .route("/", get(fetch_all).put(add))
        .route("/ratings/:product_id", get(ratings).put(rate))
}

#[typeshare]
#[derive(Serialize, Debug)]
struct Product {
    name: String,
    price: f64,
    rating: Option<i32>,
    image: Option<String>,
}

async fn fetch_all(session: Claims, state: State<AppState>) -> Result<Json<HashMap<Uuid,Product>>, ProductError> {
    let mut conn = state.pool.acquire().await.unwrap();
    let res = query!(r#"
    SELECT id, name, price, rating, image_id
    FROM products p
    LEFT JOIN product_ratings pr ON pr.product_id = p.id AND pr.user_id = $1
    "#, session.user_id
    )
    .fetch_all(&mut *conn)
    .await?;

    let product_ids = res.iter().filter_map(|product| {
        if let Some(image_id) = product.image_id {
            return Some((product.id, image_id));
        }
        None
    }).collect();

    let mut products: HashMap<Uuid, Product> = res.into_iter().map(|product| (product.id, Product{
        name: product.name,
        price: product.price,
        rating: product.rating,
        image: None
    } )).collect();

    let mut set = state.bucket_client.download_many(product_ids).await;
    while let Some(join) = set.join_next().await {
        if let Ok((product_id, data)) = join {
            if let Ok((image_id, image)) = data {
                debug!("Fetched image: {image_id}");
                debug!("Image: {image}");
                products.entry(product_id).and_modify(|product| product.image = Some(image));
            } else {
                error!("Image encoding failed");
            }
        } else {
            error!("Join failed");
        }

    }
    debug!("{products:#?}");
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

async fn add(session: Claims, state: State<AppState>, Json(body): Json<AddProduct>) -> Result<(), ProductError> {
    let image_id = match body.image {
        Some(image) => Some(state.bucket_client.upload(image).await?),
        None => None
    };

    let _id = query!(r#"
        INSERT INTO products (name, price, image_id)
        VALUES ($1, $2, $3)
        "#, body.name, body.price, image_id).execute(&state.pool).await?;

    if image_id.is_some() {
        debug!("Saved product with image");
    } else {
        debug!("Saved product without image");
    }



    Ok(())
}