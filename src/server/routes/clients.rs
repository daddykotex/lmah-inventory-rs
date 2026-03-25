use axum::{
    Router,
    extract::{Path, State},
    routing::get,
};
use maud::Markup;
use sqlx::SqlitePool;

use crate::{
    cli,
    server::{database::has_table::Table, routes::errors::AppError, templates::clients},
};

async fn list_clients(State(pool): State<SqlitePool>) -> Result<Markup, AppError> {
    let query = format!("SELECT COUNT(*) FROM {}", Table::Clients);
    let (count,): (i64,) = sqlx::query_as(&query).fetch_one(&pool).await?;

    let rendered = clients::page_clients(count);

    Ok(rendered)
}

async fn new_client(State(pool): State<SqlitePool>) -> Result<Markup, AppError> {
    Ok(clients::page_new_client())
}

async fn one_client(
    Path(client_id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> Result<Markup, AppError> {
    Ok(clients::page_one_client(client_id))
}

pub fn client_router() -> Router<SqlitePool> {
    Router::new()
        .route("/clients", get(list_clients))
        .route("/clients/new", get(new_client))
        .route("/clients/{client_id}", get(one_client))
}
