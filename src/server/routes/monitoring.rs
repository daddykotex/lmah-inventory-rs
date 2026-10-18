use axum::{Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use sqlx::SqlitePool;

use crate::server::routes::bootstrap::AppState;

async fn health(State(db_pool): State<SqlitePool>) -> impl IntoResponse {
    match sqlx::query_scalar::<_, i64>("SELECT 1")
        .fetch_one(&db_pool)
        .await
    {
        Ok(_) => (StatusCode::OK, "ok"),
        Err(err) => {
            eprintln!("Health check failed: {:?}", err);
            (StatusCode::SERVICE_UNAVAILABLE, "unavailable")
        }
    }
}

pub fn monitoring_router() -> Router<AppState> {
    Router::new().route("/monitoring/health", get(health))
}
