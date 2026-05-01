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
pub struct WebConfig {
    url: Arc<str>,
    cookie_key: Arc<str>,
    authorized_users: Arc<Vec<String>>,
}

impl WebConfig {
    pub fn new(url: String, cookie_key: String, authorized_users: Vec<String>) -> Self {
        Self {
            url: Arc::from(url),
            cookie_key: Arc::from(cookie_key),
            authorized_users: Arc::from(authorized_users),
        }
    }

    pub fn url(&self) -> Arc<str> {
        Arc::clone(&self.url)
    }

    pub fn cookie_key(&self) -> Arc<str> {
        Arc::clone(&self.cookie_key)
    }

    pub fn check_if_users_is_authorized(&self, email: &str) -> Result<()> {
        if self.authorized_users.contains(&email.to_string()) {
            return Ok(());
        }
        anyhow::bail!("Unauthorized user.")
    }
}

#[derive(Clone)]
pub struct GoogleConfig {
    oauth_client_key: Arc<str>,
    oauth_client_secret: Arc<str>,
    service_account_json_key: Option<Arc<str>>,
    bucket_name: Arc<str>,
}

impl GoogleConfig {
    pub fn new(
        oauth_client_key: String,
        oauth_client_secret: String,
        service_account_json_key: Option<String>,
        bucket_name: String,
    ) -> Self {
        Self {
            oauth_client_key: Arc::from(oauth_client_key),
            oauth_client_secret: Arc::from(oauth_client_secret),
            service_account_json_key: service_account_json_key.map(Arc::from),
            bucket_name: Arc::from(bucket_name),
        }
    }

    pub fn oauth_client_key(&self) -> Arc<str> {
        Arc::clone(&self.oauth_client_key)
    }

    pub fn oauth_client_secret(&self) -> Arc<str> {
        Arc::clone(&self.oauth_client_secret)
    }

    pub fn service_account_json_key(&self) -> Option<Arc<str>> {
        self.service_account_json_key.clone()
    }

    pub fn bucket_name(&self) -> Arc<str> {
        Arc::clone(&self.bucket_name)
    }
}

#[derive(Clone)]
pub struct PdfRocketConfig {
    api_key: Arc<str>,
}

impl PdfRocketConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key: Arc::from(api_key),
        }
    }

    pub fn api_key(&self) -> Arc<str> {
        Arc::clone(&self.api_key)
    }
}

#[derive(Clone)]
pub struct RouterConfig {
    web: WebConfig,
    google: GoogleConfig,
    pdf_rocket: PdfRocketConfig,
}

impl RouterConfig {
    pub fn new(web: WebConfig, google: GoogleConfig, pdf_rocket: PdfRocketConfig) -> Self {
        Self {
            web,
            google,
            pdf_rocket,
        }
    }

    pub fn google_oauth_client_secret(&self) -> Arc<str> {
        self.google.oauth_client_secret()
    }

    pub fn google_oauth_client_key(&self) -> Arc<str> {
        self.google.oauth_client_key()
    }

    pub fn google_service_account_json_key(&self) -> Option<Arc<str>> {
        self.google.service_account_json_key()
    }

    pub fn google_bucket_name(&self) -> Arc<str> {
        self.google.bucket_name()
    }

    pub fn external_url(&self) -> Arc<str> {
        self.web.url()
    }

    pub fn cookie_key(&self) -> Arc<str> {
        self.web.cookie_key()
    }

    pub fn pdf_rocket_api_key(&self) -> Arc<str> {
        self.pdf_rocket.api_key()
    }

    pub fn check_if_users_is_authorized(&self, email: &str) -> Result<()> {
        self.web.check_if_users_is_authorized(email)
    }
}
