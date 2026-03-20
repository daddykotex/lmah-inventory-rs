use crate::server::database::has_table::HasTable;
use crate::server::models::config::ConfigRow;
use crate::server::models::events::EventRow;
use crate::server::models::product_types::ProductTypeRow;
use crate::server::{database::insert::Insertable, models::clients::ClientRow};
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
    pub events: AirtableRecords<EventFields>,
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

/// Unnecessary WithId but kept because most other tabs have one
impl From<AirtableRecord<ConfigFields>> for WithId<ConfigRow> {
    fn from(record: AirtableRecord<ConfigFields>) -> Self {
        WithId {
            airtable_id: String::from("_N/A_"),
            row: ConfigRow {
                key: record.fields.key,
                value: record.fields.value,
                config_type: record.fields.config_type,
                created_at: record.created_time.clone(),
                updated_at: record.created_time,
            },
        }
    }
}

/// Wrapper for rows that need Airtable ID mapping
#[derive(Debug)]
pub struct WithId<T> {
    pub row: T,
    pub airtable_id: String,
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
    pub clients: Vec<WithId<ClientRow>>,
    pub product_types: Vec<ProductTypeRow>,
    pub events: Vec<WithId<EventRow>>,
}

pub async fn load_from_export(data: AirtableExport) -> Result<ToInsert> {
    let config = load_config_from_export(data.config).await?;
    let clients = load_clients_from_export(data.clients).await?;
    let product_types = load_product_types_from_export(data.product_types).await?;
    let events = load_events_from_export(data.events).await?;
    return Ok(ToInsert {
        config,
        clients,
        product_types,
        events,
    });
}

/// Load config records
async fn load_config_from_export(data: AirtableRecords<ConfigFields>) -> Result<Vec<ConfigRow>> {
    let mut rows = Vec::new();
    for (idx, record) in data.records.into_iter().enumerate() {
        validate_config_type(&record.fields.config_type).with_context(|| {
            format!("Invalid config type in record {} (id: {})", idx, record.id)
        })?;

        let row = WithId::<ConfigRow>::from(record).row;

        rows.push(row);
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

impl From<AirtableRecord<ClientFields>> for WithId<ClientRow> {
    fn from(record: AirtableRecord<ClientFields>) -> Self {
        WithId {
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
) -> Result<Vec<WithId<ClientRow>>> {
    let mut rows = Vec::new();
    for (idx, record) in data.records.into_iter().enumerate() {
        validate_client_fields(&record.fields).with_context(|| {
            format!("Invalid client data in record {} (id: {})", idx, record.id)
        })?;

        rows.push(WithId::<ClientRow>::from(record));
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

/// Unnecessary WithId but kept because most other tabs have one
impl From<AirtableRecord<ProductTypeFields>> for WithId<ProductTypeRow> {
    fn from(record: AirtableRecord<ProductTypeFields>) -> Self {
        WithId {
            airtable_id: String::from("_N/A_"),
            row: ProductTypeRow {
                name: record.fields.name,
            },
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

        let row = WithId::<ProductTypeRow>::from(record).row;

        rows.push(row);
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

/// Event fields from Airtable
#[derive(Debug, Deserialize)]
pub struct EventFields {
    #[serde(rename = "Nom")]
    name: String,
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "Type")]
    event_type: String,
}

impl From<AirtableRecord<EventFields>> for WithId<EventRow> {
    fn from(record: AirtableRecord<EventFields>) -> Self {
        WithId {
            airtable_id: record.id,
            row: EventRow {
                name: record.fields.name,
                event_type: record.fields.event_type,
                date: record.fields.date,
                created_at: record.created_time.clone(),
                updated_at: record.created_time,
            },
        }
    }
}

/// Load event records from Airtable JSON export
async fn load_events_from_export(
    data: AirtableRecords<EventFields>,
) -> Result<Vec<WithId<EventRow>>> {
    let mut rows = Vec::new();
    for (idx, record) in data.records.into_iter().enumerate() {
        validate_event_fields(&record.fields)
            .with_context(|| format!("Invalid event data in record {} (id: {})", idx, record.id))?;

        rows.push(WithId::<EventRow>::from(record));
    }

    println!("Loaded {} event records from JSON", rows.len());
    Ok(rows)
}

fn validate_event_fields(fields: &EventFields) -> Result<()> {
    if fields.name.trim().is_empty() {
        anyhow::bail!("event name cannot be empty");
    }
    if fields.date.trim().is_empty() {
        anyhow::bail!("event date cannot be empty");
    }
    if fields.event_type.trim().is_empty() {
        anyhow::bail!("event type cannot be empty");
    }
    Ok(())
}

pub async fn load_records<R, T>(
    pool: &SqlitePool,
    data: AirtableRecords<R>,
    clear_existing: bool,
) -> Result<()>
where
    WithId<T>: From<AirtableRecord<R>>,
    T: HasTable,
    T: Insertable,
{
    let mut converted = Vec::new();
    for r in data.records {
        converted.push(WithId::<T>::from(r));
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
            for with_id in converted {
                let maybe_id = with_id.row.insert_one(&mut tx).await?;
                match maybe_id {
                    Some(id) => {
                        insert_airtable_id(&mut tx, T::table_name(), id, with_id.airtable_id).await?;
                    }
                    None => {}
                }
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

async fn insert_airtable_id(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    table_name: &'static str,
    id: i64,
    airtable_id: String,
) -> Result<()> {
    // Insert mapping into airtable_mapping table
    sqlx::query(
        "INSERT INTO airtable_mapping (table_name, airtable_id, db_id)
             VALUES (?, ?, ?)",
    )
    .bind(table_name)
    .bind(&airtable_id)
    .bind(id)
    .execute(&mut **tx)
    .await
    .with_context(|| {
        format!(
            "Failed to insert airtable mapping for {}: id = {}, airtable_id = {}",
            table_name, id, airtable_id
        )
    })?;
    Ok(())
}
