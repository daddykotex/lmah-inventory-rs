use anyhow::Result;
use axum::{Router, response::Redirect, routing::get};
use toasty::Db;
use tower_http::services::ServeDir;

use crate::server::routes::{
    clients::client_router, events::event_router, factures::facture_router,
};

async fn redirect_to_factures() -> Redirect {
    Redirect::to("/factures")
}

pub async fn setup_routes() -> Result<Router<Db>> {
    Ok(Router::new()
        .merge(client_router())
        .merge(event_router())
        .merge(facture_router())
        .route("/", get(redirect_to_factures))
        .nest_service("/static", ServeDir::new("static")))
}
