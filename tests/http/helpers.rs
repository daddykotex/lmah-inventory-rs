use anyhow::Result;
use axum::body::{Body, to_bytes};
use axum::http::Request;
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;
use std::str::FromStr;

/// Create an in-memory test database with migrations applied
pub async fn create_test_db() -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")?
        .foreign_keys(true)
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    // Run migrations from project root
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

/// Build a GET request
pub fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .method("GET")
        .body(Body::empty())
        .unwrap()
}

/// Simple URL encoding for form values
fn url_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

/// Build a POST request with form data
pub fn post_form_request(uri: &str, form_data: &[(&str, &str)]) -> Request<Body> {
    let body = form_data
        .iter()
        .map(|(k, v)| format!("{}={}", k, url_encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    Request::builder()
        .uri(uri)
        .method("POST")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap()
}

/// Extract response body as string
pub async fn body_to_string(body: Body) -> String {
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

/// Extract redirect location header
pub fn get_redirect_location(response: &axum::response::Response) -> Option<String> {
    response
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
}
