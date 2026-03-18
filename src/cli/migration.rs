use crate::server::models::clients::ClientRow;
use crate::server::models::config::ConfigRow;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;

/// Root JSON structure matching Airtable export format
#[derive(Debug, Deserialize)]
pub struct AirtableConfigExport {
    records: Vec<AirtableConfigRecord>,
}

/// Individual Airtable record
#[derive(Debug, Deserialize)]
pub struct AirtableConfigRecord {
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

impl From<AirtableConfigRecord> for ConfigRow {
    fn from(record: AirtableConfigRecord) -> Self {
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

    let parsed: serde_json::Value = serde_json::from_str(&json_content)
        .context("Failed to parse JSON")?;

    let config_section = parsed.get("config")
        .context("Missing 'config' key in JSON")?;
    
    let export: AirtableConfigExport = serde_json::from_value(config_section.clone())
        .context("Failed to parse config section")?;

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

/// Client fields from Airtable
#[derive(Debug, Deserialize)]
pub struct ClientFields {
    #[serde(rename = "Prenom")]
    first_name: String,
    #[serde(rename = "Nom")]
    last_name: String,
    #[serde(rename = "Rue")]
    street: Option<String>,
    #[serde(rename = "Ville")]
    city: Option<String>,
    #[serde(rename = "Téléphone")]
    phone1: String,
    #[serde(rename = "Téléphone #2")]
    phone2: Option<String>,
}

/// Individual Airtable client record
#[derive(Debug, Deserialize)]
pub struct AirtableClientRecord {
    id: String,
    #[serde(rename = "createdTime")]
    created_time: String,
    fields: ClientFields,
}

/// Root JSON structure for clients export
#[derive(Debug, Deserialize)]
pub struct AirtableClientsExport {
    records: Vec<AirtableClientRecord>,
}

impl From<AirtableClientRecord> for ClientRow {
    fn from(record: AirtableClientRecord) -> Self {
        ClientRow {
            airtable_id: record.id,
            first_name: record.fields.first_name,
            last_name: record.fields.last_name,
            street: record.fields.street,
            city: record.fields.city,
            phone1: record.fields.phone1,
            phone2: record.fields.phone2,
            created_at: record.created_time.clone(),
            updated_at: record.created_time,
        }
    }
}

/// Load client records from Airtable JSON export
pub async fn load_clients_from_json(json_path: &std::path::Path) -> Result<Vec<ClientRow>> {
    let json_content = fs::read_to_string(json_path)
        .with_context(|| format!("Failed to read JSON file: {}", json_path.display()))?;

    let parsed: serde_json::Value = serde_json::from_str(&json_content)
        .context("Failed to parse JSON")?;

    let clients_section = parsed.get("clients")
        .context("Missing 'clients' key in JSON")?;

    let export: AirtableClientsExport = serde_json::from_value(clients_section.clone())
        .context("Failed to parse clients section")?;

    let mut rows = Vec::new();
    for (idx, record) in export.records.into_iter().enumerate() {
        validate_client_fields(&record.fields).with_context(|| {
            format!("Invalid client data in record {} (id: {})", idx, record.id)
        })?;

        rows.push(ClientRow::from(record));
    }

    println!("Loaded {} client records from JSON", rows.len());
    Ok(rows)
}

fn validate_client_fields(fields: &ClientFields) -> Result<()> {
    if fields.first_name.trim().is_empty() {
        anyhow::bail!("first_name cannot be empty");
    }
    if fields.last_name.trim().is_empty() {
        anyhow::bail!("last_name cannot be empty");
    }
    if fields.phone1.trim().is_empty() {
        anyhow::bail!("phone1 cannot be empty");
    }
    Ok(())
}
