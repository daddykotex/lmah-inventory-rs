use sqlx::prelude::FromRow;

/// Database row structure for config table
#[derive(Debug, FromRow)]
pub struct ConfigRow {
    pub id: i64,
    pub key: String,
    pub value: String,
    pub config_type: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug)]
pub struct ConfigInsert {
    pub key: String,
    pub value: String,
    pub config_type: String,
}

pub struct NoteTemplate {
    pub note_type: String,
    pub key: String,
    pub value: String,
}

pub struct ExtraLargeAmounts {
    pub wedding: i64,
    pub others: i64,
}
