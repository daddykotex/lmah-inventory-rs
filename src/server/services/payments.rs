use anyhow::{Context, Result};
use sqlx::SqlitePool;

use crate::server::utils::money::parse_money;
use crate::server::{
    database::{insert::Insertable, select::Selectable},
    models::{
        facture_items::FactureItemRow,
        factures::FactureRow,
        payments::{PaymentForm, PaymentInsert, PaymentRow},
        refunds::RefundRow,
    },
};

/// Validate payment amount against facture balance
/// For create: amount must not exceed balance
/// For update: amount must not exceed (balance + old_amount)
async fn validate_payment_amount(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    facture_id: i64,
    amount: i64,
    existing_payment_id: Option<i64>,
) -> Result<()> {
    // Get facture and related data to calculate balance
    let facture =
        FactureRow::select_one(facture_id, &mut **tx)
            .await?
            .ok_or(anyhow::Error::msg(format!(
                "Facture with id {} not found",
                facture_id
            )))?;

    let facture_items = FactureItemRow::select_all_for_facture(facture_id, &mut **tx).await?;
    let payments = PaymentRow::select_all_for_facture(facture_id, &mut **tx).await?;
    let refunds = RefundRow::select_all_for_facture(facture_id, &mut **tx).await?;

    // Calculate balance (copied from factures service)
    let total: i64 = if let Some(fixed) = facture.fixed_total {
        fixed
    } else {
        facture_items
            .iter()
            .map(|item| item.price.unwrap_or(0) * item.quantity)
            .sum()
    };

    const TPS_RATE: f64 = 5.0;
    const TVQ_RATE: f64 = 9.975;
    let tps: i64 = (TPS_RATE / 100.0 * (total as f64)).round() as i64;
    let tvq: i64 = (TVQ_RATE / 100.0 * (total as f64)).round() as i64;
    let tax_total = total + tps + tvq;

    // Calculate current payments total, excluding the payment being updated
    let total_payments: i64 = payments
        .iter()
        .filter(|p| existing_payment_id != Some(p.id))
        .map(|p| p.amount)
        .sum();

    let total_refunds: i64 = refunds.iter().map(|r| r.amount).sum();
    let balance = tax_total - total_payments + total_refunds;

    if amount > balance {
        anyhow::bail!(
            "Payment amount ({}) exceeds facture balance ({})",
            amount,
            balance
        );
    }

    Ok(())
}

/// Insert a new payment
pub async fn insert_payment(pool: &SqlitePool, facture_id: i64, form: PaymentForm) -> Result<i64> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let amount = parse_money(&form.amount).map_err(anyhow::Error::msg)?;

    // Validate amount
    validate_payment_amount(&mut tx, facture_id, amount, None).await?;

    let to_insert = PaymentInsert {
        facture_id,
        amount,
        date: form.date,
        payment_type: form.payment_type,
        cheque_number: form.cheque_number,
    };

    let inserted_id = to_insert
        .insert_one(&mut tx)
        .await?
        .expect("An ID should be generated for a new Payment");

    tx.commit().await.context("Failed to commit transaction")?;

    Ok(inserted_id)
}

/// Update an existing payment
pub async fn update_payment(pool: &SqlitePool, payment_id: i64, form: PaymentForm) -> Result<u64> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    // Verify payment exists and get facture_id
    let maybe_payment: Option<PaymentRow> = sqlx::query_as("SELECT * FROM payments WHERE id = ?")
        .bind(payment_id)
        .fetch_optional(&mut *tx)
        .await
        .with_context(|| format!("Failed to fetch payment {}", payment_id))?;

    let payment = maybe_payment.ok_or(anyhow::Error::msg(format!(
        "Payment with id {} not found",
        payment_id
    )))?;

    let amount = parse_money(&form.amount).map_err(anyhow::Error::msg)?;

    // Validate amount (excluding this payment from balance calculation)
    validate_payment_amount(&mut tx, payment.facture_id, amount, Some(payment_id)).await?;

    let result = sqlx::query(
        "UPDATE payments SET
            amount = ?, date = ?, payment_type = ?, cheque_number = ?,
            updated_at = datetime('now')
         WHERE id = ?",
    )
    .bind(amount)
    .bind(form.date)
    .bind(form.payment_type)
    .bind(form.cheque_number)
    .bind(payment_id)
    .execute(&mut *tx)
    .await
    .with_context(|| format!("Failed to update payment {}", payment_id))?;

    tx.commit().await.context("Failed to commit transaction")?;

    Ok(result.rows_affected())
}

/// Delete a payment
pub async fn delete_payment(pool: &SqlitePool, payment_id: i64) -> Result<u64> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    // Verify payment exists
    let maybe_payment: Option<PaymentRow> = sqlx::query_as("SELECT * FROM payments WHERE id = ?")
        .bind(payment_id)
        .fetch_optional(&mut *tx)
        .await
        .with_context(|| format!("Failed to fetch payment {}", payment_id))?;

    maybe_payment.ok_or(anyhow::Error::msg(format!(
        "Payment with id {} not found",
        payment_id
    )))?;

    let result = sqlx::query("DELETE FROM payments WHERE id = ?")
        .bind(payment_id)
        .execute(&mut *tx)
        .await
        .with_context(|| format!("Failed to delete payment {}", payment_id))?;

    tx.commit().await.context("Failed to commit transaction")?;

    Ok(result.rows_affected())
}
