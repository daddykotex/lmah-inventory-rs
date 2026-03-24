use anyhow::Result;
use axum::Router;
use sqlx::SqlitePool;
use tower_http::services::ServeDir;

use crate::server::routes::clients::client_router;

pub async fn setup_routes() -> Result<Router<SqlitePool>> {
    Ok(Router::new()
        .merge(client_router())
        .nest_service("/static", ServeDir::new("static")))
}
