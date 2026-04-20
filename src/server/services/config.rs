use anyhow::{Ok, Result};
use sqlx::{Executor, Sqlite};

use crate::server::{
    database::select::Selectable,
    models::config::{ConfigRow, ExtraLargeAmounts, NoteTemplate},
};

pub async fn load_note_templates<'c, E>(executor: E) -> Result<Vec<NoteTemplate>>
where
    E: Executor<'c, Database = Sqlite>,
{
    let rows = ConfigRow::select_all(executor).await?;

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

pub async fn load_extra_large_amount<'c, E>(executor: E) -> Result<ExtraLargeAmounts>
where
    E: Executor<'c, Database = Sqlite>,
{
    let rows = ConfigRow::select_all(executor).await?;

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

pub async fn load_seamstresses<'c, E>(executor: E) -> Result<Vec<String>>
where
    E: Executor<'c, Database = Sqlite>,
{
    let rows = ConfigRow::select_all(executor).await?;

    let seamstresses = rows
        .into_iter()
        .filter(|row| row.config_type == "couturiere")
        .map(|row| row.value)
        .collect();

    Ok(seamstresses)
}

pub async fn load_clauses<'c, E>(executor: E) -> Result<Vec<String>>
where
    E: Executor<'c, Database = Sqlite>,
{
    let rows = ConfigRow::select_all(executor).await?;

    let clauses = rows
        .into_iter()
        .filter(|row| row.config_type == "clause-facture")
        .map(|row| row.value)
        .collect();

    Ok(clauses)
}

pub async fn load_signatures<'c, E>(executor: E) -> Result<Vec<String>>
where
    E: Executor<'c, Database = Sqlite>,
{
    let rows = ConfigRow::select_all(executor).await?;

    let signatures = rows
        .into_iter()
        .filter(|row| row.config_type == "signature-facture")
        .map(|row| row.value)
        .collect();

    Ok(signatures)
}

pub async fn load_event_types<'c, E>(e: E) -> Result<Vec<String>>
where
    E: Executor<'c, Database = Sqlite>,
{
    let rows = ConfigRow::select_all(e).await?;

    let event_types = rows
        .into_iter()
        .filter(|row| row.config_type == "event-type")
        .map(|row| row.value)
        .collect();

    Ok(event_types)
}
