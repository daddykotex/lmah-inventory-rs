use sqlx::prelude::FromRow;

/// Database row structure for payments table
#[derive(Debug, FromRow)]
pub struct PaymentRow {
    pub facture_id: i64, // Required FK to factures
    pub amount: i64,     // Amount in cents
    pub date: String,
    pub payment_type: String,
    pub cheque_number: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug)]
pub struct PaymentView {
    pub facture_id: i64, // Required FK to factures
    pub amount: i64,     // Amount in cents
    pub date: String,
    pub payment_type: String,
    pub cheque_number: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<PaymentRow> for PaymentView {
    fn from(value: PaymentRow) -> Self {
        PaymentView {
            facture_id: value.facture_id,
            amount: value.amount,
            date: value.date,
            payment_type: value.payment_type,
            cheque_number: value.cheque_number,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
