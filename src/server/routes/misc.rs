use axum::{Router, extract::State, response::IntoResponse, routing::get};
use axum_streams::{CsvStreamFormat, StreamBodyAs};
use futures_util::StreamExt;
use maud::Markup;
use sqlx::SqlitePool;

use crate::server::{
    routes::{RouterConfig, bootstrap::AppState, errors::AppError},
    services::{
        admin::{PaymentReportRecord, load_all_factures, load_payment_reports_data},
        config::{load_event_types, load_extra_large_amount},
    },
    templates::misc::{page_admin, page_help, page_wait},
};
async fn wait() -> Result<Markup, AppError> {
    Ok(page_wait())
}

async fn help(State(db_pool): State<SqlitePool>) -> Result<Markup, AppError> {
    let event_types = load_event_types(&db_pool).await?;
    let extra = load_extra_large_amount(&db_pool).await?;
    Ok(page_help(event_types, extra))
}

async fn admin(State(db_pool): State<SqlitePool>) -> Result<Markup, AppError> {
    let factures = load_all_factures(&db_pool).await?;
    Ok(page_admin(factures))
}

async fn payment_report(
    State(db_pool): State<SqlitePool>,
    State(config): State<RouterConfig>,
) -> impl IntoResponse {
    let data = load_payment_reports_data(&db_pool, &config.external_url());
    // StreamBodyAs::csv(data)
    // use the above instead of below
    let res = data.collect::<Vec<PaymentReportRecord>>().await; // remove
    StreamBodyAs::new(
        CsvStreamFormat::default(),
        futures_util::stream::iter(res).map(Ok::<PaymentReportRecord, axum::Error>),
    )
}

pub fn misc_router() -> Router<AppState> {
    Router::new()
        .route("/wait", get(wait))
        .route("/help", get(help))
        .route("/admin", get(admin))
        .route("/admin/rapport-paiements", get(payment_report))
        .route("/admin/rapport-factures", get(admin))
}
