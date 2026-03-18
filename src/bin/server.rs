use axum::{Router, response::Html, routing::get};

async fn home() -> Html<&'static str> {
    Html("hello world")
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(home));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
