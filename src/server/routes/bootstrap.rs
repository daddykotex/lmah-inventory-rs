use axum::{Extension, Router, extract::FromRef, middleware, response::Redirect, routing::get};
use axum_extra::extract::cookie::Key;
use sqlx::SqlitePool;
use tower_http::services::ServeDir;

use crate::server::routes::{
    RouterConfig,
    auth::{UserData, auth_router},
    clients::client_router,
    events::event_router,
    factures::facture_router,
    middleware::{check_auth, inject_user_data},
};

async fn redirect_to_factures() -> Redirect {
    Redirect::to("/factures")
}

#[derive(Clone, FromRef)]
pub struct AppState {
    pub db_pool: SqlitePool,
    pub config: RouterConfig,
    pub key: Key,
}

pub fn setup_routes(db_pool: SqlitePool, config: RouterConfig) -> Router {
    let decoded_key =
        hex::decode(config.cookie_key()).expect("Unable to hex decode the cookie key");
    let key = Key::try_from(decoded_key.as_slice());
    let key = key.expect("Unable to load cookie key");
    let app_state = AppState {
        db_pool,
        config,
        key,
    };
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
    router
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            inject_user_data,
        ))
        .with_state(app_state)
        .layer(Extension(user_data))
}
