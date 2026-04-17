use axum::{Router, routing::get};
use maud::{Markup};

use crate::server::{routes::{bootstrap::AppState, errors::AppError}, templates::misc::page_signin};

#[derive(Clone)]
pub struct UserData;

async fn sign_in() -> Result<Markup, AppError> {
    Ok(page_signin())
}

pub fn auth_router() -> Router<AppState> {
    Router::new()
    .route("/signin", get(sign_in))
}
