/// Database row structure for events table
/// Note: Airtable ID mapping is stored in the airtable_mapping table
#[derive(Debug)]
pub struct EventRow {
    pub name: String,
    pub event_type: String,
    pub date: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Wrapper for rows that need Airtable ID mapping
#[derive(Debug)]
pub struct EventRowWithId {
    pub row: EventRow,
    pub airtable_id: String,
}
