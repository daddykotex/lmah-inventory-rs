use anyhow::{Context, Result};
use axum::{Extension, Router, extract::FromRef, middleware, response::Redirect, routing::get};
use axum_extra::extract::cookie::Key;
use google_cloud_auth::{
    credentials::{Credentials, service_account::Builder},
    signer::Signer,
};
use google_cloud_storage::client::Storage;
use reqwest::Client;
use sqlx::SqlitePool;
use tower_http::services::ServeDir;

use crate::server::routes::{
    RouterConfig,
    auth::{UserData, auth_router},
    clients::client_router,
    events::event_router,
    factures::facture_router,
    middleware::{check_auth, inject_user_data},
    misc::misc_router,
};

async fn redirect_to_factures() -> Redirect {
    Redirect::to("/factures")
}

#[derive(Clone, FromRef)]
pub struct AppState {
    pub db_pool: SqlitePool,
    pub config: RouterConfig,
    pub key: Key, // Used by axum-private-cookies
    pub storage: Storage,
    pub signer: Signer,
    pub http_client: Client,
}

async fn prepare_app_state(db_pool: SqlitePool, config: RouterConfig) -> Result<AppState> {
    // Load Cookie Key (for signed and encrypted cookies)
    let decoded_key = hex::decode(config.cookie_key().clone().to_string())
        .expect("Unable to hex decode the cookie key");
    let key = Key::try_from(decoded_key.as_slice());
    let key = key.expect("Unable to load cookie key");

    // Load google credentials
    let json_service_account_key: serde_json::Value =
        serde_json::from_str(&config.google_service_account_json_key())
            .context("Unable to load JSON from environment variable")?;
    let google_credentials: Credentials = Builder::new(json_service_account_key.clone()).build()?;
    let storage = Storage::builder()
        .with_credentials(google_credentials)
        .build()
        .await?;

    let signer = Builder::new(json_service_account_key.clone()).build_signer()?;

    let http_client = Client::new();

    Ok(AppState {
        db_pool,
        config,
        key,
        storage,
        signer,
        http_client,
    })
}

pub async fn setup_routes(db_pool: SqlitePool, config: RouterConfig) -> Router {
    let app_state = prepare_app_state(db_pool, config)
        .await
        .expect("Unable to prepare AppState");
    let user_data: Option<UserData> = None;

    let authed_routes = Router::new()
        .merge(misc_router())
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
