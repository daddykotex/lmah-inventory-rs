use std::sync::Arc;

use anyhow::Result;

pub mod auth;
pub mod bootstrap;
pub mod clients;
pub mod errors;
pub mod events;
pub mod factures;
pub mod middleware;
pub mod misc;
pub mod redirect;

#[derive(Clone)]
pub struct RouterConfig {
    external_url: Arc<str>,
    google_oauth_client_key: Arc<str>,
    google_oauth_client_secret: Arc<str>,
    google_service_account_json_key: Arc<str>,
    google_bucket_name: Arc<str>,
    cookie_key: Arc<str>,
    pdf_rocket_api_key: Arc<str>,
    authorized_users: Arc<Vec<String>>,
}

impl RouterConfig {
    pub fn new(
        external_url: String,
        google_oauth_client_key: String,
        google_oauth_client_secret: String,
        google_service_account_json_key: String,
        google_bucket_name: String,
        cookie_key: String,
        pdf_rocket_api_key: String,
        authorized_users: Vec<String>,
    ) -> Self {
        Self {
            external_url: Arc::from(external_url),
            google_oauth_client_key: Arc::from(google_oauth_client_key),
            google_oauth_client_secret: Arc::from(google_oauth_client_secret),
            google_service_account_json_key: Arc::from(google_service_account_json_key),
            google_bucket_name: Arc::from(google_bucket_name),
            cookie_key: Arc::from(cookie_key),
            pdf_rocket_api_key: Arc::from(pdf_rocket_api_key),
            authorized_users: Arc::from(authorized_users),
        }
    }

    pub fn google_oauth_client_secret(&self) -> Arc<str> {
        Arc::clone(&self.google_oauth_client_secret)
    }

    pub fn google_oauth_client_key(&self) -> Arc<str> {
        Arc::clone(&self.google_oauth_client_key)
    }

    pub fn google_service_account_json_key(&self) -> Arc<str> {
        Arc::clone(&self.google_service_account_json_key)
    }

    pub fn google_bucket_name(&self) -> Arc<str> {
        Arc::clone(&self.google_bucket_name)
    }

    pub fn external_url(&self) -> Arc<str> {
        Arc::clone(&self.external_url)
    }

    pub fn cookie_key(&self) -> Arc<str> {
        Arc::clone(&self.cookie_key)
    }

    pub fn pdf_rocket_api_key(&self) -> Arc<str> {
        Arc::clone(&self.pdf_rocket_api_key)
    }

    pub fn check_if_users_is_authorized(&self, email: &str) -> Result<()> {
        if self.authorized_users.contains(&email.to_string()) {
            return Ok(());
        }
        anyhow::bail!("Unauthorized user.")
    }
}
