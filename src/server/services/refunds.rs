use anyhow::{Context, Result};
use sqlx::SqlitePool;

use crate::server::utils::money::parse_money;
use crate::server::{
    database::{insert::Insertable, select::Selectable},
    models::{
        facture_items::FactureItemRow,
        factures::FactureRow,
        payments::PaymentRow,
        refunds::{RefundForm, RefundInsert, RefundRow},
    },
};

/// Validate refund amount against total_refundable
/// For create: amount must not exceed total_refundable
/// For update: amount must not exceed (total_refundable + old_amount)
async fn validate_refund_amount(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    facture_id: i64,
    amount: i64,
    existing_refund_id: Option<i64>,
) -> Result<()> {
    // Get facture and related data to calculate total_refundable
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

    // Calculate total_refundable (copied from factures service)
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
    let _tax_total = total + tps + tvq;

    let total_payments: i64 = payments.iter().map(|p| p.amount).sum();

    // Calculate current refunds total, excluding the refund being updated
    let total_refunds: i64 = refunds
        .iter()
        .filter(|r| existing_refund_id.map_or(true, |id| r.id != id))
        .map(|r| r.amount)
        .sum();

    let total_refundable = total_payments - total_refunds;

    if amount > total_refundable {
        anyhow::bail!(
            "Refund amount ({}) exceeds total refundable ({})",
            amount,
            total_refundable
        );
    }

    Ok(())
}

/// Insert a new refund
pub async fn insert_refund(pool: &SqlitePool, facture_id: i64, form: RefundForm) -> Result<i64> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let amount = parse_money(&form.amount).map_err(anyhow::Error::msg)?;

    // Validate amount
    validate_refund_amount(&mut tx, facture_id, amount, None).await?;

    let to_insert = RefundInsert {
        facture_id,
        amount,
        date: form.date,
        refund_type: form.refund_type,
        cheque_number: form.cheque_number,
    };

    let inserted_id = to_insert
        .insert_one(&mut tx)
        .await?
        .expect("An ID should be generated for a new Refund");

    tx.commit().await.context("Failed to commit transaction")?;

    Ok(inserted_id)
}

/// Update an existing refund
pub async fn update_refund(pool: &SqlitePool, refund_id: i64, form: RefundForm) -> Result<u64> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    // Verify refund exists and get facture_id
    let maybe_refund: Option<RefundRow> = sqlx::query_as("SELECT * FROM refunds WHERE id = ?")
        .bind(refund_id)
        .fetch_optional(&mut *tx)
        .await
        .with_context(|| format!("Failed to fetch refund {}", refund_id))?;

    let refund = maybe_refund.ok_or(anyhow::Error::msg(format!(
        "Refund with id {} not found",
        refund_id
    )))?;

    let amount = parse_money(&form.amount).map_err(anyhow::Error::msg)?;

    // Validate amount (excluding this refund from total_refundable calculation)
    validate_refund_amount(&mut tx, refund.facture_id, amount, Some(refund_id)).await?;

    let result = sqlx::query(
        "UPDATE refunds SET
            amount = ?, date = ?, refund_type = ?, cheque_number = ?,
            updated_at = datetime('now')
         WHERE id = ?",
    )
    .bind(amount)
    .bind(form.date)
    .bind(form.refund_type)
    .bind(form.cheque_number)
    .bind(refund_id)
    .execute(&mut *tx)
    .await
    .with_context(|| format!("Failed to update refund {}", refund_id))?;

    tx.commit().await.context("Failed to commit transaction")?;

    Ok(result.rows_affected())
}

/// Delete a refund
pub async fn delete_refund(pool: &SqlitePool, refund_id: i64) -> Result<u64> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    // Verify refund exists
    let maybe_refund: Option<RefundRow> = sqlx::query_as("SELECT * FROM refunds WHERE id = ?")
        .bind(refund_id)
        .fetch_optional(&mut *tx)
        .await
        .with_context(|| format!("Failed to fetch refund {}", refund_id))?;

    maybe_refund.ok_or(anyhow::Error::msg(format!(
        "Refund with id {} not found",
        refund_id
    )))?;

    let result = sqlx::query("DELETE FROM refunds WHERE id = ?")
        .bind(refund_id)
        .execute(&mut *tx)
        .await
        .with_context(|| format!("Failed to delete refund {}", refund_id))?;

    tx.commit().await.context("Failed to commit transaction")?;

    Ok(result.rows_affected())
}
