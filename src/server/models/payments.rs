use serde::Deserialize;
use sqlx::prelude::FromRow;

/// Database row structure for payments table
#[derive(Debug, FromRow)]
pub struct PaymentRow {
    pub id: i64,
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
    pub id: i64,
    pub facture_id: i64, // Required FK to factures
    pub amount: i64,     // Amount in cents
    pub date: String,
    pub payment_type: String,
    pub cheque_number: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug)]
pub struct PaymentInsert {
    pub facture_id: i64, // Required FK to factures
    pub amount: i64,     // Amount in cents
    pub date: String,
    pub payment_type: String,
    pub cheque_number: Option<String>,
}

impl From<PaymentRow> for PaymentView {
    fn from(value: PaymentRow) -> Self {
        PaymentView {
            id: value.id,
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

pub struct PreCalculatedPayment {
    pub amount_ratio: i64,
    pub is_alteration: bool,
    pub tax_total: i64,
    pub balance: i64,
}

impl PreCalculatedPayment {
    pub fn label(&self, amount: i64) -> String {
        format!("{}% de {}", self.amount_ratio, amount)
    }

    pub fn calculate(&self) -> Option<i64> {
        if self.tax_total == self.balance {
            if self.is_alteration && self.balance < 50 {
                Some(self.balance)
            } else {
                let amount = self.tax_total as f64 * self.amount_ratio as f64 / 100_f64;
                Some(amount as i64)
            }
        } else {
            None
        }
    }
}

// Form structure for POST endpoints
#[derive(Deserialize, Debug)]
pub struct PaymentForm {
    pub amount: i64,
    pub date: String,
    #[serde(rename = "payment-type")]
    pub payment_type: String,
    #[serde(rename = "cheque-number")]
    pub cheque_number: Option<String>,
}
