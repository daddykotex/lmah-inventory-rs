use crate::server::models::config::ConfigRow;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;

/// Root JSON structure matching Airtable export format
#[derive(Debug, Deserialize)]
pub struct AirtableExport {
    records: Vec<AirtableRecord>,
}

/// Individual Airtable record
#[derive(Debug, Deserialize)]
pub struct AirtableRecord {
    id: String,
    #[serde(rename = "createdTime")]
    created_time: String,
    fields: ConfigFields,
}

/// Config fields from Airtable
#[derive(Debug, Deserialize)]
pub struct ConfigFields {
    #[serde(rename = "Clé")]
    key: String,
    #[serde(rename = "Valeur")]
    value: String,
    #[serde(rename = "Type")]
    config_type: String,
}

impl From<AirtableRecord> for ConfigRow {
    fn from(record: AirtableRecord) -> Self {
        ConfigRow {
            key: record.fields.key,
            value: record.fields.value,
            config_type: record.fields.config_type,
            created_at: record.created_time.clone(),
            updated_at: record.created_time,
        }
    }
}

/// Load config records from Airtable JSON export
pub async fn load_config_from_json(json_path: &std::path::Path) -> Result<Vec<ConfigRow>> {
    let json_content = fs::read_to_string(json_path)
        .with_context(|| format!("Failed to read JSON file: {}", json_path.display()))?;

    let export: AirtableExport =
        serde_json::from_str(&json_content).context("Failed to parse JSON")?;

    let mut rows = Vec::new();
    for (idx, record) in export.records.into_iter().enumerate() {
        validate_config_type(&record.fields.config_type).with_context(|| {
            format!("Invalid config type in record {} (id: {})", idx, record.id)
        })?;

        rows.push(ConfigRow::from(record));
    }

    println!("Loaded {} config records from JSON", rows.len());
    Ok(rows)
}

fn validate_config_type(config_type: &str) -> Result<()> {
    const VALID_TYPES: &[&str] = &[
        "clause-facture",
        "signature-facture",
        "formule-type-location",
        "formule-type-alteration",
        "formule-type-robes",
        "formule-type-autres",
        "event-type",
        "extra-taille-forte",
        "couturiere",
    ];

    if VALID_TYPES.contains(&config_type) {
        Ok(())
    } else {
        anyhow::bail!(
            "Invalid config type: '{}'. Must be one of: {:?}",
            config_type,
            VALID_TYPES
        )
    }
}
