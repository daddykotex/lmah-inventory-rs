use crate::server::models::clients::ClientRow;
use crate::server::models::config::ConfigRow;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use sqlx::sqlite::SqlitePool;
use sqlx::{Sqlite, Transaction};
use std::fs;

// ============================================================================
// Generic Types for Airtable Import
// ============================================================================

/// Generic Airtable record structure
#[derive(Debug, Deserialize)]
pub struct AirtableRecord<T> {
    pub id: String,
    #[serde(rename = "createdTime")]
    pub created_time: String,
    pub fields: T,
}

/// Generic table structure containing records
#[derive(Debug, Deserialize)]
pub struct AirtableRecords<T> {
    pub records: Vec<AirtableRecord<T>>,
}

/// Trait for types that can be imported from Airtable into the database
#[async_trait]
pub trait ImportableRecord: Sized + Send {
    /// The table name in the database
    fn table_name() -> &'static str;

    /// Check if table exists in the database
    async fn verify_table_exists(pool: &SqlitePool) -> Result<()> {
        let table = Self::table_name();
        let query = format!(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='{}'",
            table
        );
        let result: (i64,) = sqlx::query_as(&query)
            .fetch_one(pool)
            .await
            .with_context(|| format!("Failed to verify {} table", table))?;

        if result.0 == 0 {
            anyhow::bail!("{} table does not exist. Run migration.sql first.", table);
        }
        Ok(())
    }

    /// Count existing records in the table
    async fn check_existing_count(pool: &SqlitePool) -> Result<i64> {
        let table = Self::table_name();
        let query = format!("SELECT COUNT(*) FROM {}", table);
        let (count,): (i64,) = sqlx::query_as(&query).fetch_one(pool).await?;
        Ok(count)
    }

    /// Clear all records from the table
    async fn clear_table(tx: &mut Transaction<'_, Sqlite>) -> Result<()> {
        let table = Self::table_name();
        let query = format!("DELETE FROM {}", table);
        sqlx::query(&query)
            .execute(&mut **tx)
            .await
            .with_context(|| format!("Failed to clear {} table", table))?;
        Ok(())
    }

    /// Insert a single record into the database
    async fn insert_record(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<()>;

    /// Get a display name for the record (for error messages)
    fn display_name(&self) -> String;
}

/// Generic function to import records into the database
pub async fn import_records<T>(
    pool: &SqlitePool,
    records: Vec<T>,
    clear_existing: bool,
) -> Result<()>
where
    T: ImportableRecord,
{
    // Verify table exists
    T::verify_table_exists(pool).await?;

    // Check existing records
    let existing_count = T::check_existing_count(pool).await?;

    if existing_count > 0 && !clear_existing {
        anyhow::bail!(
            "{} table already contains {} records. Use --clear-existing flag to clear and reload.",
            T::table_name(),
            existing_count
        );
    }

    // Begin transaction
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    // Clear existing if requested
    if clear_existing && existing_count > 0 {
        T::clear_table(&mut tx).await?;
        println!(
            "Cleared {} existing {} records",
            existing_count,
            T::table_name()
        );
    }

    // Insert all records
    let record_count = records.len();
    for record in records {
        record
            .insert_record(&mut tx)
            .await
            .with_context(|| format!("Failed to insert {}", record.display_name()))?;
    }

    // Commit transaction
    tx.commit().await.context("Failed to commit transaction")?;

    println!("Inserted {} new {} records", record_count, T::table_name());
    Ok(())
}

// ============================================================================
// Airtable Export Structure
// ============================================================================

/// Root JSON structure matching Airtable export format
#[derive(Debug, Deserialize)]
pub struct AirtableExport {
    pub config: AirtableRecords<ConfigFields>,
    pub clients: AirtableRecords<ClientFields>,
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

impl From<AirtableRecord<ConfigFields>> for ConfigRow {
    fn from(record: AirtableRecord<ConfigFields>) -> Self {
        ConfigRow {
            key: record.fields.key,
            value: record.fields.value,
            config_type: record.fields.config_type,
            created_at: record.created_time.clone(),
            updated_at: record.created_time,
        }
    }
}

/// Load data from JSON file
pub async fn load_data(json_path: &std::path::Path) -> Result<AirtableExport> {
    let json_content = fs::read_to_string(json_path)
        .with_context(|| format!("Failed to read JSON file: {}", json_path.display()))?;

    let export: AirtableExport =
        serde_json::from_str(&json_content).context("Failed to parse JSON")?;

    Ok(export)
}

pub struct ToInsert {
    pub config: Vec<ConfigRow>,
    pub clients: Vec<ClientRow>,
}

pub async fn load_from_export(data: AirtableExport) -> Result<ToInsert> {
    let config = load_config_from_export(data.config).await?;
    let clients = load_clients_from_export(data.clients).await?;
    return Ok(ToInsert {
        config: config,
        clients: clients,
    });
}

/// Load config records
async fn load_config_from_export(data: AirtableRecords<ConfigFields>) -> Result<Vec<ConfigRow>> {
    let mut rows = Vec::new();
    for (idx, record) in data.records.into_iter().enumerate() {
        validate_config_type(&record.fields.config_type).with_context(|| {
            format!("Invalid config type in record {} (id: {})", idx, record.id)
        })?;

        rows.push(ConfigRow::from(record));
    }
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

impl From<AirtableRecord<ClientFields>> for ClientRow {
    fn from(record: AirtableRecord<ClientFields>) -> Self {
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
async fn load_clients_from_export(data: AirtableRecords<ClientFields>) -> Result<Vec<ClientRow>> {
    let mut rows = Vec::new();
    for (idx, record) in data.records.into_iter().enumerate() {
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
