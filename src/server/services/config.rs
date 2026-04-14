use anyhow::{Ok, Result};

use crate::server::models::config::{ExtraLargeAmounts, NoteTemplate};

pub async fn load_note_templates(
    _tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<Vec<NoteTemplate>> {
    // TODO read db
    Ok(vec![NoteTemplate {
        note_type: "TODO".to_string(),
        key: "key".to_string(),
        value: "value".to_string(),
    }])
}

pub async fn load_extra_large_amount(
    _tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<ExtraLargeAmounts> {
    // TODO read db
    Ok(ExtraLargeAmounts {
        wedding: -100,
        others: -1000,
    })
}

pub async fn load_seamstresses(
    _tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<Vec<String>> {
    // TODO read db
    Ok(vec![String::from("Rachel")])
}

pub async fn load_clauses(_tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<Vec<String>> {
    // TODO read db
    Ok(Vec::from([
        "clause 1".into(),
        "clause 2".into(),
        "clause 3".into(),
    ]))
}

pub async fn load_signatures(_tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<Vec<String>> {
    // TODO read db
    Ok(Vec::from(["sign 1".into(), "sign 2".into()]))
}
