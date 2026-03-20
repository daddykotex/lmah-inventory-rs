/// Database row structure for events table
#[derive(Debug)]
pub struct EventRow {
    pub name: String,
    pub event_type: String,
    pub date: String,
    pub created_at: String,
    pub updated_at: String,
}
