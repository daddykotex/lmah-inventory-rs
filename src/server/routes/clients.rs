use axum::{
    Form, Router,
    extract::{Path, State},
    response::Redirect,
    routing::{get, post},
};
use maud::Markup;
use toasty::Db;

use crate::server::{
    models::clients::{Client, ClientForm, ClientView},
    routes::errors::AppError,
    services::clients::{insert_client, update_client},
    templates::clients,
};

async fn list_clients(State(mut db): State<Db>) -> Result<Markup, AppError> {
    let clients = Client::all().exec(&mut db).await?;
    let client_views = clients.into_iter().map(ClientView::from).collect();
    let rendered = clients::page_clients(client_views);

    Ok(rendered)
}

async fn new_client() -> Result<Markup, AppError> {
    Ok(clients::page_new_client())
}

async fn one_client(
    State(mut db): State<Db>,
    Path(client_id): Path<u64>,
) -> Result<Markup, AppError> {
    let maybe_client = Client::filter_by_id(client_id)
        .first()
        .exec(&mut db)
        .await?;
    let client = maybe_client.ok_or(anyhow::Error::msg(format!(
        "Client with id {} not found",
        client_id
    )))?;
    let client_view = ClientView::from(client);
    Ok(clients::page_one_client(client_view))
}

async fn update_one_client(
    State(mut db): State<Db>,
    Path(client_id): Path<u64>,
    Form(update): Form<ClientForm>,
) -> Result<Redirect, AppError> {
    update_client(&mut db, client_id, update).await?;
    let url = format!("/clients/{}?success=true", client_id);
    Ok(Redirect::to(&url))
}

async fn create_one_client(
    State(mut db): State<Db>,
    Form(create): Form<ClientForm>,
) -> Result<Redirect, AppError> {
    let id = insert_client(&mut db, create).await?;
    let url = format!("/clients/{}?success=true", id);
    Ok(Redirect::to(&url))
}

pub fn client_router() -> Router<Db> {
    Router::new()
        .route("/clients", get(list_clients))
        .route("/clients/new", get(new_client))
        .route("/clients/{client_id}", get(one_client))
        .route("/clients/new", post(create_one_client))
        .route("/clients/{client_id}/update", post(update_one_client))
}
