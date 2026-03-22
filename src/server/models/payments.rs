/// Database row structure for payments table
#[derive(Debug)]
pub struct PaymentRow {
    pub facture_id: i64, // Required FK to factures
    pub amount: i64,     // Amount in cents
    pub date: String,
    pub payment_type: String,
    pub cheque_number: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
