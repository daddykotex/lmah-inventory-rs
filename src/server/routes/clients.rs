use axum::{Router, extract::State, response::{Html, }, routing::get};
use sqlx::{SqlitePool};

use crate::server::{database::has_table::Table, routes::errors::AppError};

async fn list_clients(State(pool): State<SqlitePool>) -> Result<Html<String>, AppError> {
    let query = format!("SELECT COUNT(*) FROM {}", Table::Clients);
    let (count,): (i64,) = sqlx::query_as(&query).fetch_one(&pool).await?;


    Ok(Html(format!("hello from {} clients", count)))
}

pub fn client_router() -> Router<SqlitePool> {
    Router::new().route("/clients", get(list_clients))
}
