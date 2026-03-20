/// Database row structure for clients table
/// Note: Airtable ID mapping is stored in the airtable_mapping table
#[derive(Debug)]
pub struct ClientRow {
    pub first_name: String,
    pub last_name: String,
    pub street: Option<String>,
    pub city: Option<String>,
    pub phone1: String,
    pub phone2: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Wrapper for rows that need Airtable ID mapping
#[derive(Debug)]
pub struct ClientRowWithId {
    pub row: ClientRow,
    pub airtable_id: String,
}
