use anyhow::{Ok, Result};

use crate::server::models::config::{ConfigRow, ExtraLargeAmounts, NoteTemplate};

pub async fn load_note_templates(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<Vec<NoteTemplate>> {
    let rows = ConfigRow::select_all(tx).await?;

    let templates = rows
        .into_iter()
        .filter(|row| row.config_type.starts_with("formule-type"))
        .map(|row| NoteTemplate {
            note_type: row.config_type,
            key: row.key,
            value: row.value,
        })
        .collect();

    Ok(templates)
}

pub async fn load_extra_large_amount(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<ExtraLargeAmounts> {
    let rows = ConfigRow::select_all(tx).await?;

    let mut wedding = None;
    let mut others = None;

    for row in rows {
        if row.config_type == "extra-taille-forte" {
            if row.key == "robes-de-mariées" {
                wedding = Some(row.value.parse::<i64>()?);
            } else if row.key == "autres-robes" {
                others = Some(row.value.parse::<i64>()?);
            }
        }
    }

    Ok(ExtraLargeAmounts {
        wedding: wedding.ok_or_else(|| anyhow::anyhow!("Missing wedding extra large amount"))?,
        others: others.ok_or_else(|| anyhow::anyhow!("Missing others extra large amount"))?,
    })
}

pub async fn load_seamstresses(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<Vec<String>> {
    let rows = ConfigRow::select_all(tx).await?;

    let seamstresses = rows
        .into_iter()
        .filter(|row| row.config_type == "couturiere")
        .map(|row| row.value)
        .collect();

    Ok(seamstresses)
}

pub async fn load_clauses(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<Vec<String>> {
    let rows = ConfigRow::select_all(tx).await?;

    let clauses = rows
        .into_iter()
        .filter(|row| row.config_type == "clause-facture")
        .map(|row| row.value)
        .collect();

    Ok(clauses)
}

pub async fn load_signatures(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<Vec<String>> {
    let rows = ConfigRow::select_all(tx).await?;

    let signatures = rows
        .into_iter()
        .filter(|row| row.config_type == "signature-facture")
        .map(|row| row.value)
        .collect();

    Ok(signatures)
}
