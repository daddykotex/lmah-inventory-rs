use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::PrivateCookieJar;

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

pub async fn inject_user_data(
    cookie_jar: PrivateCookieJar,
    mut request: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, AppError> {
    if cookie_jar.get("user").is_some() {
        request.extensions_mut().insert(Some(UserData {}));
    }
    Ok(next.run(request).await)
}
