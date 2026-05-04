use axum::body::Bytes;
use axum_extra::extract::cookie::Key;
use google_cloud_auth::{
    credentials::anonymous::Builder as AnonymousCredentials,
    signer::{Signer, SigningError, SigningProvider},
};
use google_cloud_storage::client::Storage;
use lmah_inventory_rs::server::routes::{
    GoogleConfig, PdfRocketConfig, RouterConfig, WebConfig, bootstrap::AppState,
};
use reqwest::Client;
use sqlx::SqlitePool;

#[cfg(test)]
pub mod clients;
#[cfg(test)]
pub mod events;
#[cfg(test)]
pub mod factures;

#[derive(Debug)]
struct NoopSigner;

impl SigningProvider for NoopSigner {
    async fn client_email(&self) -> Result<String, SigningError> {
        Ok("test@test.iam.gserviceaccount.com".to_string())
    }

    async fn sign(&self, _content: &[u8]) -> Result<Bytes, SigningError> {
        Ok(Bytes::new())
    }
}

pub async fn make_state(pool: SqlitePool) -> AppState {
    let web_config = WebConfig::new(
        "".to_string(),
        "".to_string(),
        vec!["user@test.com".to_string()],
    );

    let google_config = GoogleConfig::new("".to_string(), "".to_string(), None, "".to_string());

    let pdf_rocket_config = PdfRocketConfig::new("".to_string());

    let config = RouterConfig::new(web_config, google_config, pdf_rocket_config);

    #[expect(
        clippy::let_underscore_must_use,
        reason = "Test file so we can ignore if provider setup is already done."
    )]
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    let storage = Storage::builder()
        .with_credentials(AnonymousCredentials::new().build())
        .build()
        .await
        .expect("failed to build storage");
    AppState {
        db_pool: pool,
        config,
        key: Key::generate(),
        http_client: Client::new(),
        signer: Signer::from(NoopSigner),
        storage,
    }
}
