use anyhow::Result;
use axum::Router;
use sqlx::SqlitePool;

use crate::server::routes::clients::client_router;

pub async fn setup_routes() -> Result<Router<SqlitePool>> {
    Ok(Router::new().merge(client_router()))
}
