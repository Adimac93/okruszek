mod errors;

use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::routing::{get, MethodRouter, put};
use serde::{Serialize, Deserialize};
use sqlx::{PgPool, query, query_as};
use typeshare::typeshare;
use uuid::Uuid;
use crate::AppState;
use crate::routes::auth::session::Claims;
use crate::routes::products::comments::errors::CommentError;

pub fn method_router() -> MethodRouter<AppState> {
    get(product_comments).put(comment).delete(delete_comment)
}

#[typeshare]
#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
struct AddComment {
    content: String,
}

async fn comment(claims: Claims, State(pool): State<PgPool>, Path(product_id): Path<Uuid>, Json(body): Json<AddComment>) -> Result<(), CommentError > {
    query!(r#"
    INSERT INTO product_comments (product_id, user_id, content)
    VALUES ($1, $2, $3)
    "#, product_id, claims.user_id, body.content).execute(&pool).await?;

    Ok(())
}

#[typeshare]
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
struct Comment {
    author: String,
    content: String,
}

async fn product_comments(claims: Claims, State(pool): State<PgPool>, Path(product_id): Path<Uuid>) -> Result<Json<Vec<Comment>>, CommentError> {
    let res = query_as!(Comment, r#"
    SELECT users.username AS author, content
    FROM product_comments
    JOIN users ON users.id = product_comments.user_id
    WHERE product_id = $1
    "#, product_id).fetch_all(&pool).await?;

    Ok(Json(res))
}

async fn delete_comment(claims: Claims, State(pool): State<PgPool>, Path(comment_id): Path<Uuid> ) -> Result<(), CommentError> {
    query!(r#"
    DELETE FROM product_comments
    WHERE user_id = $1 AND id = $2
    "#, claims.user_id, comment_id).fetch_all(&pool).await?;

    Ok(())
}