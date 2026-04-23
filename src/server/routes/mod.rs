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
    external_url: String,
    google_oauth_client_key: String,
    google_oauth_client_secret: String,
    google_service_account_json_key: String,
    google_bucket_name: String,
    cookie_key: String,
    pdf_rocket_api_key: String,
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
    ) -> Self {
        Self {
            external_url,
            google_oauth_client_key,
            google_oauth_client_secret,
            google_service_account_json_key,
            google_bucket_name,
            cookie_key,
            pdf_rocket_api_key,
        }
    }

    pub fn google_oauth_client_secret(&self) -> &str {
        &self.google_oauth_client_secret
    }

    pub fn google_oauth_client_key(&self) -> &str {
        &self.google_oauth_client_key
    }

    pub fn google_service_account_json_key(&self) -> &str {
        &self.google_service_account_json_key
    }

    pub fn google_bucket_name(&self) -> &str {
        &self.google_bucket_name
    }

    pub fn external_url(&self) -> &str {
        &self.external_url
    }

    pub fn cookie_key(&self) -> &str {
        &self.cookie_key
    }

    pub fn pdf_rocket_api_key(&self) -> &str {
        &self.pdf_rocket_api_key
    }
}
