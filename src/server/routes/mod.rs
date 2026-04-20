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
    oauth_client_key: String,
    oauth_client_secret: String,
    cookie_key: String,
}

impl RouterConfig {
    pub fn new(
        external_url: String,
        oauth_client_key: String,
        oauth_client_secret: String,
        cookie_key: String,
    ) -> Self {
        Self {
            external_url,
            oauth_client_key,
            oauth_client_secret,
            cookie_key,
        }
    }

    pub fn oauth_client_secret(&self) -> &str {
        &self.oauth_client_secret
    }

    pub fn oauth_client_key(&self) -> &str {
        &self.oauth_client_key
    }

    pub fn external_url(&self) -> &str {
        &self.external_url
    }

    pub fn cookie_key(&self) -> &str {
        &self.cookie_key
    }
}
