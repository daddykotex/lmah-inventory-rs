use crate::server::database::has_table::HasTable;
use crate::server::models::config::ConfigRow;
use crate::server::models::product_types::ProductTypeRow;
use crate::server::{
    database::insert::Insertable,
    models::clients::{ClientRow, ClientRowWithId},
};
use anyhow::{Context, Result};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::fs;

/// Root JSON structure matching Airtable export format
#[derive(Debug, Deserialize)]
pub struct AirtableExport {
    pub config: AirtableRecords<ConfigFields>,
    pub clients: AirtableRecords<ClientFields>,
    pub product_types: AirtableRecords<ProductTypeFields>,
}

/// Table in the JSON
#[derive(Debug, Deserialize)]
pub struct AirtableRecords<T> {
    pub records: Vec<AirtableRecord<T>>,
}

/// Individual Airtable record
#[derive(Debug, Deserialize)]
pub struct AirtableRecord<T> {
    id: String,
    #[serde(rename = "createdTime")]
    created_time: String,
    fields: T,
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
    pub clients: Vec<ClientRowWithId>,
    pub product_types: Vec<ProductTypeRow>,
}

pub async fn load_from_export(data: AirtableExport) -> Result<ToInsert> {
    let config = load_config_from_export(data.config).await?;
    let clients = load_clients_from_export(data.clients).await?;
    let product_types = load_product_types_from_export(data.product_types).await?;
    return Ok(ToInsert {
        config,
        clients,
        product_types,
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

impl From<AirtableRecord<ClientFields>> for ClientRowWithId {
    fn from(record: AirtableRecord<ClientFields>) -> Self {
        ClientRowWithId {
            airtable_id: record.id,
            row: ClientRow {
                first_name: record.fields.first_name,
                last_name: record.fields.last_name,
                street: record.fields.street,
                city: record.fields.city,
                phone1: record.fields.phone1,
                phone2: record.fields.phone2,
                created_at: record.created_time.clone(),
                updated_at: record.created_time,
            },
        }
    }
}

/// Load client records from Airtable JSON export
async fn load_clients_from_export(
    data: AirtableRecords<ClientFields>,
) -> Result<Vec<ClientRowWithId>> {
    let mut rows = Vec::new();
    for (idx, record) in data.records.into_iter().enumerate() {
        validate_client_fields(&record.fields).with_context(|| {
            format!("Invalid client data in record {} (id: {})", idx, record.id)
        })?;

        rows.push(ClientRowWithId::from(record));
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

/// Product type fields from Airtable
#[derive(Debug, Deserialize)]
pub struct ProductTypeFields {
    #[serde(rename = "Name")]
    name: String,
}

impl From<AirtableRecord<ProductTypeFields>> for ProductTypeRow {
    fn from(record: AirtableRecord<ProductTypeFields>) -> Self {
        ProductTypeRow {
            name: record.fields.name,
        }
    }
}

/// Load product_types records from Airtable JSON export
async fn load_product_types_from_export(
    data: AirtableRecords<ProductTypeFields>,
) -> Result<Vec<ProductTypeRow>> {
    let mut rows = Vec::new();
    for (idx, record) in data.records.into_iter().enumerate() {
        validate_product_type_fields(&record.fields).with_context(|| {
            format!(
                "Invalid product_type data in record {} (id: {})",
                idx, record.id
            )
        })?;

        rows.push(ProductTypeRow::from(record));
    }

    println!("Loaded {} product_type records from JSON", rows.len());
    Ok(rows)
}

fn validate_product_type_fields(fields: &ProductTypeFields) -> Result<()> {
    if fields.name.trim().is_empty() {
        anyhow::bail!("product_type name cannot be empty");
    }
    Ok(())
}

pub async fn load_records<R, T>(
    pool: &SqlitePool,
    data: AirtableRecords<R>,
    clear_existing: bool,
) -> Result<()>
where
    T: From<AirtableRecord<R>>,
    T: HasTable,
    T: Insertable,
{
    let mut converted = Vec::new();
    for r in data.records {
        converted.push(T::from(r));
    }

    let count_records = count_records(pool, &T::table_name()).await?;
    match count_records {
        Some(count) => {
            if !clear_existing {
                anyhow::bail!("There");
            }

            if count > 0 {
                clear_table(pool, T::table_name()).await?;
            }

            let mut tx: sqlx::Transaction<'_, sqlx::Sqlite> =
                pool.begin().await.context("Failed to begin transaction")?;
            for row in converted {
                row.insert_one(&mut tx).await?;
            }
            tx.commit().await.context("Failed to commit transaction")?;
            Ok(())
        }
        None => todo!(),
    }
}

async fn count_records(pool: &SqlitePool, table_name: &'static str) -> Result<Option<i64>> {
    let result: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?")
            .bind(table_name)
            .fetch_one(pool)
            .await
            .context("Failed to verify config table")?;

    if result.0 == 0 {
        return Ok(None);
    }

    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM config")
        .fetch_one(pool)
        .await?;
    Ok(Some(count))
}

async fn clear_table(pool: &SqlitePool, table_name: &'static str) -> Result<()> {
    sqlx::query(&format!("DELETE FROM {}", table_name))
        .execute(pool)
        .await
        .context("Failed to clear clients table")?;
    Ok(())
}
