use axum::{Extension, Router, extract::FromRef, middleware, response::Redirect, routing::get};
use sqlx::SqlitePool;
use tower_http::services::ServeDir;

use crate::server::routes::{
    auth::{UserData, auth_router},
    clients::client_router,
    events::event_router,
    factures::facture_router,
    middleware::check_auth,
};

async fn redirect_to_factures() -> Redirect {
    Redirect::to("/factures")
}

#[derive(Clone, FromRef)]
pub struct AppState {
    pub db_pool: SqlitePool,
}

pub fn setup_routes(db_pool: SqlitePool) -> Router {
    let app_state = AppState { db_pool };
    let user_data: Option<UserData> = None;

    let authed_routes = Router::new()
        .merge(client_router())
        .merge(event_router())
        .merge(facture_router())
        .route("/", get(redirect_to_factures))
        .route_layer(middleware::from_fn(check_auth));

    let unauthed_routes = Router::new()
        .merge(auth_router())
        .nest_service("/static", ServeDir::new("static"));

    let router = Router::new().merge(authed_routes).merge(unauthed_routes);
    router.with_state(app_state).layer(Extension(user_data))
}
