/// Database row structure for config table
#[derive(Debug)]
pub struct ConfigRow {
    pub key: String,
    pub value: String,
    pub config_type: String,
    pub created_at: String,
    pub updated_at: String,
}
