use axum::{
    Form, Router,
    extract::{Path, State},
    response::Redirect,
    routing::{get, post},
};
use maud::Markup;
use sqlx::SqlitePool;

use crate::server::{
    models::clients::{ClientForm, ClientView},
    routes::errors::AppError,
    services::clients::{insert_client, select_all, select_one, update_client},
    templates::clients,
};

async fn list_clients(State(pool): State<SqlitePool>) -> Result<Markup, AppError> {
    let clients = select_all(&pool).await?;
    let client_views = clients.into_iter().map(ClientView::from).collect();
    let rendered = clients::page_clients(client_views);

    Ok(rendered)
}

async fn new_client() -> Result<Markup, AppError> {
    Ok(clients::page_new_client())
}

async fn one_client(
    State(pool): State<SqlitePool>,
    Path(client_id): Path<i64>,
) -> Result<Markup, AppError> {
    let maybe_client = select_one(&pool, client_id).await?;
    let client = maybe_client.ok_or(anyhow::Error::msg(format!(
        "Client with id {} not found",
        client_id
    )))?;
    let client_view = ClientView::from(client);
    Ok(clients::page_one_client(client_view))
}

async fn update_one_client(
    State(pool): State<SqlitePool>,
    Path(client_id): Path<i64>,
    Form(update): Form<ClientForm>,
) -> Result<Redirect, AppError> {
    update_client(&pool, client_id, update).await?;
    let url = format!("/clients/{}?success=true", client_id);
    Ok(Redirect::to(&url))
}

async fn create_one_client(
    State(pool): State<SqlitePool>,
    Form(create): Form<ClientForm>,
) -> Result<Redirect, AppError> {
    let id = insert_client(&pool, create).await?;
    let url = format!("/clients/{}?success=true", id);
    Ok(Redirect::to(&url))
}

pub fn client_router() -> Router<SqlitePool> {
    Router::new()
        .route("/clients", get(list_clients))
        .route("/clients/new", get(new_client))
        .route("/clients/{client_id}", get(one_client))
        .route("/clients/new", post(create_one_client))
        .route("/clients/{client_id}/update", post(update_one_client))
}
