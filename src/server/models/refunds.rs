/// Refund model with Toasty ORM
#[derive(Debug, toasty::Model)]
pub struct Refund {
    #[key]
    #[auto]
    id: u64,

    #[index]
    facture_id: u64,
    #[belongs_to(key = facture_id, references = id)]
    facture: toasty::BelongsTo<crate::server::models::factures::Facture>,

    amount: i64,     // Amount in cents
    date: String,
    refund_type: String,
    cheque_number: Option<String>,
    created_at: String,
    updated_at: String,
}

/// Database row structure for refunds table (kept for migration)
#[derive(Debug)]
pub struct RefundRow {
    pub id: i64,
    pub facture_id: i64, // Required FK to factures
    pub amount: i64,     // Amount in cents
    pub date: String,
    pub refund_type: String,
    pub cheque_number: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug)]
pub struct RefundInsert {
    pub facture_id: u64, // Required FK to factures
    pub amount: i64,     // Amount in cents
    pub date: String,
    pub refund_type: String,
    pub cheque_number: Option<String>,
}

#[derive(Debug)]
pub struct RefundView {
    pub id: u64,
    pub facture_id: u64, // Required FK to factures
    pub amount: i64,     // Amount in cents
    pub date: String,
    pub refund_type: String,
    pub cheque_number: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<RefundRow> for RefundView {
    fn from(value: RefundRow) -> Self {
        RefundView {
            id: value.id as u64,
            facture_id: value.facture_id as u64,
            amount: value.amount,
            date: value.date,
            refund_type: value.refund_type,
            cheque_number: value.cheque_number,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<Refund> for RefundView {
    fn from(value: Refund) -> Self {
        RefundView {
            id: value.id,
            facture_id: value.facture_id,
            amount: value.amount,
            date: value.date,
            refund_type: value.refund_type,
            cheque_number: value.cheque_number,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
