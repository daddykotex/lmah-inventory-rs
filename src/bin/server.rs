use axum::Router;
use clap::Parser;
use lmah_inventory_rs::server::{
    database::connect_to_url,
    routes::{RouterConfig, bootstrap::setup_routes},
};
use tokio::net::TcpListener;

/// Options for starting the server
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct ServerConfig {
    /// Location of the SQLite database
    #[arg(short, long, env = "DATABASE_URL")]
    db_url: String,

    /// Port of the HTTP server
    #[arg(short, long, default_value = "3000")]
    port: u16,

    /// Key used to encrypt/decrypt cookies
    #[arg(long, env = "LMAH_COOKIE_KEY")]
    lmah_cookie_key: String,

    /// OAuth key for Google OAuth2 flow
    #[arg(long, env = "LMAH_GOOGLE_OAUTH_KEY")]
    lmah_google_oauth_key: String,

    /// OAuth secret for Google OAuth2 flow
    #[arg(long, env = "LMAH_GOOGLE_OAUTH_SECRET")]
    lmah_google_oauth_secret: String,

    /// Google service account credentials (file upload)
    #[arg(long, env = "LMAH_GOOGLE_CREDENTIALS")]
    lmah_google_credentials: String,

    /// GCP Storage bucket name
    #[arg(long, env = "LMAH_GOOGLE_BUCKET_NAME")]
    lmah_google_bucket_name: String,

    /// OAuth secret for Google OAuth2 flow
    #[arg(long, env = "LMAH_EXTERNAL_URL")]
    lmah_external_url: String,

    /// PDF Rocket API key
    #[arg(long, env = "LMAH_PDF_ROCKET_API_KEY")]
    lmah_pdf_rocket_api_key: String,

    /// Authorized users: email1@test.com,email2@test.com
    #[arg(long, env = "LMAH_AUTHORIZED_USERS")]
    lmah_authorized_users: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let config = ServerConfig::parse();

    let pool = connect_to_url(&config.db_url).await?;

    let authorized_users: Vec<String> = config
        .lmah_authorized_users
        .split(",")
        .map(String::from)
        .collect();

    let router_config = RouterConfig::new(
        config.lmah_external_url,
        config.lmah_google_oauth_key,
        config.lmah_google_oauth_secret,
        config.lmah_google_credentials,
        config.lmah_google_bucket_name,
        config.lmah_cookie_key,
        config.lmah_pdf_rocket_api_key,
        authorized_users,
    );
    let app: Router = setup_routes(pool, router_config).await;

    let mut listenfd = listenfd::ListenFd::from_env();
    match listenfd.take_tcp_listener(0)? {
        Some(_listener) => {
            _listener.set_nonblocking(true)?;
            let listener = TcpListener::from_std(_listener)?;
            axum::serve(listener, app).await?;
        }
        None => {
            let listener =
                tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port)).await?;
            axum::serve(listener, app).await?;
        }
    };
    Ok(())
}
