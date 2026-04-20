use anyhow::Result;
use sqlx::SqlitePool;

pub async fn check_if_users_is_authorized(pool: &SqlitePool, email: &str) -> Result<()> {
    sqlx::query("SELECT * FROM users WHERE email = ?")
        .bind(email)
        .fetch_one(pool)
        .await?;
    Ok(())
}
