use std::{collections::HashMap, time::Duration};

use anyhow::Context;
use axum::{
    Extension, Router,
    extract::{Query, State},
    http::{Response, StatusCode},
    response::{IntoResponse, Redirect},
    routing::{get, post},
};
use axum_extra::extract::{
    PrivateCookieJar,
    cookie::{Cookie, SameSite},
};
use maud::Markup;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, RevocationUrl, Scope, TokenResponse, TokenUrl,
    basic::BasicClient, reqwest,
};
use time::OffsetDateTime;

use crate::server::{
    routes::{RouterConfig, bootstrap::AppState, errors::AppError},
    templates::misc::page_signin,
};

#[derive(Clone)]
pub struct UserData;

fn make_auth_cookie<'a>(
    name: String,
    value: String,
    expires_in: Option<OffsetDateTime>,
) -> Cookie<'a> {
    let mut cookie = Cookie::new(name, value);
    cookie.set_secure(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_http_only(true);
    cookie.set_path("/");
    if let Some(ex) = expires_in {
        cookie.set_expires(ex);
    }
    cookie
}

async fn sign_in(Query(params): Query<HashMap<String, String>>) -> Result<Markup, AppError> {
    let url = params
        .get("redirect_url")
        .map(|a| a.as_str())
        .unwrap_or("/");
    Ok(page_signin(url))
}
async fn do_sign_in(
    Extension(user_data): Extension<Option<UserData>>,
    Query(mut params): Query<HashMap<String, String>>,
    State(config): State<RouterConfig>,
    cookie_jar: PrivateCookieJar,
) -> Result<impl IntoResponse, AppError> {
    if user_data.is_some() {
        // check if already authenticated
        return Ok(Redirect::to("/").into_response());
    }

    let client = get_client(
        config.external_url(),
        config.google_oauth_client_key(),
        config.google_oauth_client_secret(),
    )?;

    let return_url = params
        .remove("return_url")
        .unwrap_or_else(|| "/".to_string());

    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/userinfo.email".to_string(),
        ))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    let expires_in = OffsetDateTime::now_utc() + Duration::from_mins(5);
    let state_cookie = make_auth_cookie(
        "state".to_string(),
        csrf_state.secret().to_string(),
        Some(expires_in),
    );
    let pkce_verifier_cookie = make_auth_cookie(
        "pkce_verifier".to_string(),
        pkce_code_verifier.secret().to_string(),
        Some(expires_in),
    );
    let url_cookie = make_auth_cookie("redirect_url".to_string(), return_url, Some(expires_in));

    Ok((
        cookie_jar
            .add(state_cookie)
            .add(pkce_verifier_cookie)
            .add(url_cookie),
        Redirect::to(authorize_url.as_str()),
    )
        .into_response())
}

async fn complete_sign_in(
    Query(mut params): Query<HashMap<String, String>>,
    State(config): State<RouterConfig>,
    mut cookie_jar: PrivateCookieJar,
) -> Result<impl IntoResponse, AppError> {
    let current_state = cookie_jar.get("state");
    let pkce_verifier = cookie_jar.get("pkce_verifier");
    let redirect_url = cookie_jar.get("redirect_url");

    let r: Result<Response<_>, AppError> = match (current_state, pkce_verifier, redirect_url) {
        (Some(current_state), Some(pkce_verifier), redirect_url) => {
            let state = CsrfToken::new(
                params
                    .remove("state")
                    .ok_or(anyhow::Error::msg("OAuth: back from ISP without state"))?,
            );
            let code = AuthorizationCode::new(
                params
                    .remove("code")
                    .ok_or(anyhow::Error::msg("OAuth: back from ISP without code"))?,
            );

            if current_state.value_trimmed() != state.secret().as_str() {
                return Ok((
                    StatusCode::UNAUTHORIZED,
                    "Missing the state or the pkce verifier".to_string(),
                )
                    .into_response());
            }

            let pkce_code_verifier =
                PkceCodeVerifier::new(pkce_verifier.value_trimmed().to_string());

            let http_client = reqwest::ClientBuilder::new()
                // Following redirects opens the client up to SSRF vulnerabilities.
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .expect("Client should build");

            let client = get_client(
                config.external_url(),
                config.google_oauth_client_key(),
                config.google_oauth_client_secret(),
            )?;
            let token_response = client
                .exchange_code(code)
                .set_pkce_verifier(pkce_code_verifier)
                .request_async(&http_client)
                .await
                .context("OAuth: exchange_code failure")?;
            let access_token = token_response.access_token().secret();

            // Get user info from Google
            let url = "https://www.googleapis.com/oauth2/v2/userinfo?oauth_token=".to_owned()
                + access_token;
            let body = reqwest::get(url)
                .await
                .context("OAuth: reqwest failed to query userinfo")?
                .text()
                .await
                .context("OAuth: reqwest received invalid userinfo")?;
            let mut body: serde_json::Value = serde_json::from_str(body.as_str())
                .context("OAuth: Serde failed to parse userinfo")?;
            let email = body["email"]
                .take()
                .as_str()
                .ok_or(anyhow::Error::msg(
                    "OAuth: Serde failed to parse email address",
                ))?
                .to_owned();
            let verified_email =
                body["verified_email"]
                    .take()
                    .as_bool()
                    .ok_or(anyhow::Error::msg(
                        "OAuth: Serde failed to parse verified_email",
                    ))?;
            if !verified_email {
                return Ok((
                    StatusCode::UNAUTHORIZED,
                    "Email address is not verified.".to_string(),
                )
                    .into_response());
            }

            config.check_if_users_is_authorized(&email)?;

            let expires_in = OffsetDateTime::now_utc() + Duration::from_hours(24);
            let add_user_cookie =
                make_auth_cookie("user".to_string(), email.to_string(), Some(expires_in));
            cookie_jar = cookie_jar.add(add_user_cookie);

            let redirect_to = redirect_url
                .as_ref()
                .map(|r| r.value_trimmed())
                .unwrap_or("/");

            Ok(Redirect::to(redirect_to).into_response().into_response())
        }
        (_, _, _) => Ok((
            StatusCode::UNAUTHORIZED,
            "Missing the state or the pkce verifier".to_string(),
        )
            .into_response()),
    };

    // Important: cookies need the same config (path, secure, etc.) to be removed.
    // That's why we re-use make_auth_cookies here
    cookie_jar = cookie_jar
        .remove(make_auth_cookie("state".to_string(), "".to_string(), None))
        .remove(make_auth_cookie(
            "pkce_verifier".to_string(),
            "".to_string(),
            None,
        ))
        .remove(make_auth_cookie(
            "redirect_url".to_string(),
            "".to_string(),
            None,
        ));

    Ok((cookie_jar, r?).into_response())
}

async fn do_sign_out(cookie_jar: PrivateCookieJar) -> impl IntoResponse {
    (cookie_jar.remove(Cookie::from("user")), Redirect::to("/"))
}

pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/signout", post(do_sign_out))
        .route("/signin", post(do_sign_in))
        .route("/signin", get(sign_in))
        .route("/signin/complete", get(complete_sign_in))
}

type CustomClient = oauth2::Client<
    oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
    oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
    oauth2::StandardTokenIntrospectionResponse<
        oauth2::EmptyExtraTokenFields,
        oauth2::basic::BasicTokenType,
    >,
    oauth2::StandardRevocableToken,
    oauth2::StandardErrorResponse<oauth2::RevocationErrorResponseType>,
    oauth2::EndpointSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointSet,
    oauth2::EndpointSet,
>;

fn get_client(external_url: &str, key: &str, secret: &str) -> Result<CustomClient, AppError> {
    let google_client_id = ClientId::new(key.to_string());
    let google_client_secret = ClientSecret::new(secret.to_string());
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .context("OAuth: invalid authorization endpoint URL")?;
    let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
        .context("OAuth: invalid token endpoint URL")?;

    let redirect_url = format!("{}/signin/complete", external_url);

    // Set up the config for the Google OAuth2 process.
    let client = BasicClient::new(google_client_id)
        .set_client_secret(google_client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(RedirectUrl::new(redirect_url).context("OAuth: invalid redirect URL")?)
        .set_revocation_url(
            RevocationUrl::new("https://oauth2.googleapis.com/revoke".to_string())
                .context("OAuth: invalid revocation endpoint URL")?,
        );
    Ok(client)
}
