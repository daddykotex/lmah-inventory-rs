use axum::{
    Form, Router,
    extract::{Path, State},
    response::Redirect,
    routing::{get, post},
};
use maud::Markup;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::server::{database::has_table::Table, routes::errors::AppError, templates::clients};

async fn list_clients(State(pool): State<SqlitePool>) -> Result<Markup, AppError> {
    let query = format!("SELECT COUNT(*) FROM {}", Table::Clients);
    let (count,): (i64,) = sqlx::query_as(&query).fetch_one(&pool).await?;

    // TODO retrieve clients using database and pass them to the page_clients function below
    let rendered = clients::page_clients(count);

    Ok(rendered)
}

async fn new_client(State(pool): State<SqlitePool>) -> Result<Markup, AppError> {
    Ok(clients::page_new_client())
}

#[derive(Deserialize, Debug)]
struct ClientForm {
    #[serde(rename = "firstname")]
    first_name: String,
    #[serde(rename = "lastname")]
    last_name: String,
    street: Option<String>,
    city: Option<String>,
    #[serde(rename = "phone1")]
    phone: String,
    phone2: Option<String>,
}

async fn one_client(
    State(pool): State<SqlitePool>,
    Path(client_id): Path<i64>,
) -> Result<Markup, AppError> {
    // TODO retrieve client using database and pass it to the page_one_client function below
    Ok(clients::page_one_client(client_id))
}

async fn update_one_client(
    State(pool): State<SqlitePool>,
    Path(client_id): Path<i64>,
    Form(update): Form<ClientForm>,
) -> Result<Redirect, AppError> {
    println!("got update {:?}", update);
    // TODO update the client using the received form
    let url = format!("/clients/{}?success=true", client_id);
    Ok(Redirect::to(&url))
}

async fn create_one_client(
    State(pool): State<SqlitePool>,
    Form(create): Form<ClientForm>,
) -> Result<Redirect, AppError> {
    println!("got create {:?}", create);
    // TODO create the client and use the generate id in the url below
    let url = format!("/clients/{}?success=true", 0);
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
