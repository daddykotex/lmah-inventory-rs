use serde::Deserialize;
use sqlx::prelude::FromRow;

/// Database row structure for refunds table
#[derive(Debug, FromRow)]
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
    pub facture_id: i64, // Required FK to factures
    pub amount: i64,     // Amount in cents
    pub date: String,
    pub refund_type: String,
    pub cheque_number: Option<String>,
}

#[derive(Debug)]
pub struct RefundView {
    pub id: i64,
    pub facture_id: i64, // Required FK to factures
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

// Form structure for POST endpoints
#[derive(Deserialize, Debug)]
pub struct RefundForm {
    pub amount: i64,
    pub date: String,
    #[serde(rename = "refund-type")]
    pub refund_type: String,
    #[serde(rename = "cheque-number")]
    pub cheque_number: Option<String>,
}
