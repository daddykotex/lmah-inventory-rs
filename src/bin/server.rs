use axum::{Router, response::Html, routing::get};
use tokio::net::TcpListener;

async fn home() -> Html<&'static str> {
    Html("hello world1")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new().route("/", get(home));

    let mut listenfd = listenfd::ListenFd::from_env();
    match listenfd.take_tcp_listener(0)? {
        Some(_listener) => {
            _listener.set_nonblocking(true)?;
            let listener = TcpListener::from_std(_listener)?;
            axum::serve(listener, app).await?;
        }
        None => {
            let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
            axum::serve(listener, app).await?;
        }
    };
    Ok(())
}
