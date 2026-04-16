/// Config model with Toasty ORM
#[derive(Debug, toasty::Model)]
#[table = "config"]
pub struct Config {
    #[key]
    #[auto]
    id: u64,

    key: String,
    value: String,
    config_type: String,
    created_at: String,
    updated_at: String,
}

/// Database row structure for config table (kept for migration)
#[derive(Debug)]
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

/// View struct for Config model
pub struct ConfigView {
    pub id: u64,
    pub key: String,
    pub value: String,
    pub config_type: String,
}

impl From<Config> for ConfigView {
    fn from(value: Config) -> Self {
        ConfigView {
            id: value.id,
            key: value.key,
            value: value.value,
            config_type: value.config_type,
        }
    }
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
