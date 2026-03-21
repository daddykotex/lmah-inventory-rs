/// Database row structure for factures table
#[derive(Debug)]
pub struct FactureRow {
    pub client_id: i64,               // Required FK to clients
    pub facture_type: Option<String>, // "Product", "Location", or "Alteration"
    pub date: Option<String>,
    pub event_id: Option<i64>,    // Optional FK to events
    pub fixed_total: Option<i64>, // Amount in cents
    pub cancelled: bool,
    pub paper_ref: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
