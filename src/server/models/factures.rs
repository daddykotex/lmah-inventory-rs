/// Facture model with Toasty ORM
#[derive(Debug, toasty::Model)]
pub struct Facture {
    #[key]
    #[auto]
    id: u64,

    #[index]
    client_id: u64,
    #[belongs_to(key = client_id, references = id)]
    pub client: toasty::BelongsTo<crate::server::models::clients::Client>,

    facture_type: Option<String>,
    date: Option<String>,

    #[index]
    event_id: Option<u64>,
    #[belongs_to(key = event_id, references = id)]
    event: toasty::BelongsTo<Option<crate::server::models::events::Event>>,

    fixed_total: Option<i64>,
    cancelled: bool,
    paper_ref: Option<String>,
    created_at: String,
    updated_at: String,

    #[has_many]
    pub facture_items: toasty::HasMany<crate::server::models::facture_items::FactureItem>,

    #[has_many]
    payments: toasty::HasMany<crate::server::models::payments::Payment>,

    #[has_many]
    refunds: toasty::HasMany<crate::server::models::refunds::Refund>,
}

/// Database row structure for factures table (kept for migration)
#[derive(Debug)]
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
    pub id: u64,
    pub client_id: u64,               // Required FK to clients
    pub facture_type: Option<String>, // "Product", "Location", or "Alteration"
    pub date: Option<String>,
    pub event_id: Option<u64>,    // Optional FK to events
    pub fixed_total: Option<i64>, // Amount in cents
    pub cancelled: bool,
    pub paper_ref: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<FactureRow> for FactureView {
    fn from(value: FactureRow) -> Self {
        FactureView {
            id: value.id as u64,
            client_id: value.client_id as u64,
            facture_type: value.facture_type,
            date: value.date,
            event_id: value.event_id.map(|id| id as u64),
            fixed_total: value.fixed_total,
            cancelled: value.cancelled,
            paper_ref: value.paper_ref,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<Facture> for FactureView {
    fn from(value: Facture) -> Self {
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

impl From<&Facture> for FactureView {
    fn from(value: &Facture) -> Self {
        FactureView {
            id: value.id,
            client_id: value.client_id,
            facture_type: value.facture_type.clone(),
            date: value.date.clone(),
            event_id: value.event_id,
            fixed_total: value.fixed_total,
            cancelled: value.cancelled,
            paper_ref: value.paper_ref.clone(),
            created_at: value.created_at.clone(),
            updated_at: value.updated_at.clone(),
        }
    }
}
