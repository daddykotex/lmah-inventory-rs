use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Redirect},
};

use crate::server::routes::{auth::UserData, errors::AppError};

pub async fn check_auth(request: Request<Body>, next: Next) -> Result<impl IntoResponse, AppError> {
    if request
        .extensions()
        .get::<Option<UserData>>()
        .ok_or(anyhow::Error::msg(
            "check_auth: extensions have no UserData",
        ))?
        .is_some()
    {
        Ok(next.run(request).await)
    } else {
        let login_url = "/signin?return_url=".to_owned() + &*request.uri().to_string();
        Ok(Redirect::to(login_url.as_str()).into_response())
    }
}
