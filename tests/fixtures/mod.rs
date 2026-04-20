use axum_extra::extract::cookie::Key;
use lmah_inventory_rs::server::routes::{RouterConfig, bootstrap::AppState};
use sqlx::SqlitePool;

#[cfg(test)]
pub mod clients;
#[cfg(test)]
pub mod events;
#[cfg(test)]
pub mod factures;

pub fn make_state(pool: SqlitePool) -> AppState {
    let config = RouterConfig::new(
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
    );
    AppState {
        db_pool: pool,
        config,
        key: Key::generate(),
    }
}
