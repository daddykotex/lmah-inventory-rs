/// Database row structure for refunds table
#[derive(Debug)]
pub struct RefundRow {
    pub facture_id: i64, // Required FK to factures
    pub amount: i64,     // Amount in cents
    pub date: String,
    pub refund_type: String,
    pub cheque_number: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
