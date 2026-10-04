pub mod browser;
pub mod discover;
pub mod session;

use url::Url;

pub fn base_url() -> Url {
    let raw = std::env::var("LMAH_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".into());
    Url::parse(&raw).expect("LMAH_BASE_URL must be a valid URL")
}
