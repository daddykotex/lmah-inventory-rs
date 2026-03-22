use std::path::PathBuf;

use axum::Router;
use clap::{Parser};
use lmah_inventory_rs::server::{database::{connect_to_path}, routes::bootstrap::setup_routes};
use tokio::net::TcpListener;


/// Options for starting the server
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Config {
    /// Location of the SQLite database
    #[arg(short, long)]
    db_path: PathBuf,

    /// Port of the HTTP server
    #[arg(short, long, default_value="3000")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::parse();

    let pool = connect_to_path(&config.db_path)
        .await?;
    let app: Router = setup_routes().await?.with_state(pool);

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
