use sqlx::prelude::FromRow;

/// Database row structure for factures table
#[derive(Debug, FromRow)]
pub struct FactureRow {
    pub id: i64,
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

#[derive(Debug)]
pub struct FactureInsert {
    pub client_id: i64,               // Required FK to clients
    pub facture_type: Option<String>, // "Product", "Location", or "Alteration"
    pub date: Option<String>,
    pub event_id: Option<i64>,    // Optional FK to events
    pub fixed_total: Option<i64>, // Amount in cents
    pub cancelled: bool,
    pub paper_ref: Option<String>,
}

#[derive(Debug)]
pub struct FactureView {
    pub id: i64,
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

impl From<FactureRow> for FactureView {
    fn from(value: FactureRow) -> Self {
        FactureView {
            id: value.id,
            client_id: value.client_id,
            facture_type: value.facture_type,
            date: value.date,
            event_id: value.event_id,
            fixed_total: value.fixed_total,
            cancelled: value.cancelled,
            paper_ref: value.paper_ref,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
