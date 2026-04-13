use crate::server::database::has_table::{HasTable, Table, TableName};
use crate::server::database::insert::Insertable;
use crate::server::models::clients::ClientInsert;
use crate::server::models::config::ConfigRow;
use crate::server::models::events::EventInsert;
use crate::server::models::facture_items::FactureItemInsert;
use crate::server::models::payments::PaymentInsert;
use crate::server::models::product_types::ProductTypeRow;
use crate::server::models::products::{ProductImageInsert, ProductInsert};
use crate::server::models::refunds::RefundInsert;
use crate::server::models::statuts::StatutInsert;
use anyhow::{Context, Result};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::fs;

/// Migration-specific: Image data extracted from Airtable attachment
/// This struct is used during migration and will be removed after migration is complete
#[derive(Debug)]
pub struct ProductImage {
    pub url: String,
    pub filename: String,
}

#[derive(Debug)]
pub struct MigrationFactureInsert {
    pub id: i64,                      // map from facture number
    pub client_id: i64,               // Required FK to clients
    pub facture_type: Option<String>, // "Product", "Location", or "Alteration"
    pub date: Option<String>,
    pub event_id: Option<i64>,    // Optional FK to events
    pub fixed_total: Option<i64>, // Amount in cents
    pub cancelled: bool,
    pub paper_ref: Option<String>,
}

impl Insertable for MigrationFactureInsert {
    async fn insert_one(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<i64>> {
        // Insert facture row
        let result = sqlx::query(
            "INSERT INTO factures (id, client_id, facture_type, date, event_id, fixed_total, cancelled, paper_ref, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))",
        )
        .bind(&self.id)
        .bind(&self.client_id)
        .bind(&self.facture_type)
        .bind(&self.date)
        .bind(&self.event_id)
        .bind(&self.fixed_total)
        .bind(if self.cancelled { 1 } else { 0 })
        .bind(&self.paper_ref)
        .execute(&mut **tx)
        .await
        .with_context(|| {
            format!(
                "Failed to insert facture {}",
                self.client_id
            )
        })?;

        // Get the database ID
        let db_id = result.last_insert_rowid();

        return Ok(Some(db_id));
    }
}

/// Migration-specific: Product with related data (types and images) for insertion
/// This struct carries all the data needed to insert a product and its related records
/// It will be removed after migration is complete
#[derive(Debug)]
pub struct ProductRowWithRelated {
    pub row: ProductInsert,
    pub airtable_id: String,
    pub product_types: Vec<String>,        // Product type names
    pub image_front: Option<ProductImage>, // Optional front image
    pub image_back: Option<ProductImage>,  // Optional back image
}

/// Root JSON structure matching Airtable export format
#[derive(Debug, Deserialize)]
pub struct AirtableExport {
    pub config: AirtableRecords<ConfigFields>,
    pub clients: AirtableRecords<ClientFields>,
    pub product_types: AirtableRecords<ProductTypeFields>,
    pub events: AirtableRecords<EventFields>,
    pub products: AirtableRecords<ProductFields>,
    pub factures: AirtableRecords<FactureFields>,
    pub facture_items: AirtableRecords<FactureItemFields>,
    pub payments: AirtableRecords<PaymentFields>,
    pub refunds: AirtableRecords<RefundFields>,
    pub statuts: AirtableRecords<StatutFields>,
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

/// Sort all records in an AirtableExport by created_time in ascending order
pub fn sort_export_by_created_time(mut export: AirtableExport) -> AirtableExport {
    export
        .config
        .records
        .sort_by(|a, b| a.created_time.cmp(&b.created_time));
    export
        .clients
        .records
        .sort_by(|a, b| a.created_time.cmp(&b.created_time));
    export
        .product_types
        .records
        .sort_by(|a, b| a.created_time.cmp(&b.created_time));
    export
        .events
        .records
        .sort_by(|a, b| a.created_time.cmp(&b.created_time));
    export
        .products
        .records
        .sort_by(|a, b| a.created_time.cmp(&b.created_time));
    export
        .factures
        .records
        .sort_by(|a, b| a.created_time.cmp(&b.created_time));
    export
        .facture_items
        .records
        .sort_by(|a, b| a.created_time.cmp(&b.created_time));
    export
        .payments
        .records
        .sort_by(|a, b| a.created_time.cmp(&b.created_time));
    export
        .refunds
        .records
        .sort_by(|a, b| a.created_time.cmp(&b.created_time));
    export
        .statuts
        .records
        .sort_by(|a, b| a.created_time.cmp(&b.created_time));
    export
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
    pub clients: Vec<WithId<ClientInsert>>,
    pub product_types: Vec<ProductTypeRow>,
    pub events: Vec<WithId<EventInsert>>,
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

impl From<AirtableRecord<ClientFields>> for WithId<ClientInsert> {
    fn from(record: AirtableRecord<ClientFields>) -> Self {
        WithId {
            airtable_id: record.id,
            row: ClientInsert {
                first_name: record.fields.first_name,
                last_name: record.fields.last_name,
                street: record.fields.street,
                city: record.fields.city,
                phone1: record.fields.phone1,
                phone2: record.fields.phone2,
            },
        }
    }
}

/// Load client records from Airtable JSON export
async fn load_clients_from_export(
    data: AirtableRecords<ClientFields>,
) -> Result<Vec<WithId<ClientInsert>>> {
    let mut rows = Vec::new();
    for (idx, record) in data.records.into_iter().enumerate() {
        validate_client_fields(&record.fields).with_context(|| {
            format!("Invalid client data in record {} (id: {})", idx, record.id)
        })?;

        rows.push(WithId::<ClientInsert>::from(record));
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

impl From<AirtableRecord<EventFields>> for WithId<EventInsert> {
    fn from(record: AirtableRecord<EventFields>) -> Self {
        WithId {
            airtable_id: record.id,
            row: EventInsert {
                name: record.fields.name,
                event_type: record.fields.event_type,
                date: record.fields.date,
            },
        }
    }
}

/// Load event records from Airtable JSON export
async fn load_events_from_export(
    data: AirtableRecords<EventFields>,
) -> Result<Vec<WithId<EventInsert>>> {
    let mut rows = Vec::new();
    for (idx, record) in data.records.into_iter().enumerate() {
        validate_event_fields(&record.fields)
            .with_context(|| format!("Invalid event data in record {} (id: {})", idx, record.id))?;

        rows.push(WithId::<EventInsert>::from(record));
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

/// Airtable attachment structure
#[derive(Debug, Deserialize)]
pub struct AirtableAttachment {
    pub id: String,
    pub url: String,
    pub filename: String,
}

/// Product fields from Airtable
#[derive(Debug, Deserialize)]
pub struct ProductFields {
    #[serde(rename = "Nom")]
    name: String,
    #[serde(rename = "Type")]
    product_types: Option<Vec<String>>, // Array of product type names
    #[serde(rename = "Prix")]
    price: Option<f64>, // Will convert to cents
    #[serde(rename = "Liquidation")]
    liquidation: Option<bool>,
    #[serde(rename = "Visible sur le site")]
    visible_on_site: Option<bool>, // Default to false if missing
    #[serde(rename = "imageSrc")]
    image_front: Option<Vec<AirtableAttachment>>,
    #[serde(rename = "imageSrc2")]
    image_back: Option<Vec<AirtableAttachment>>,
}

/// Convert dollars to cents
fn dollars_to_cents(amount: f64) -> i64 {
    (amount * 100.0).round() as i64
}

impl From<AirtableRecord<ProductFields>> for ProductRowWithRelated {
    fn from(record: AirtableRecord<ProductFields>) -> Self {
        ProductRowWithRelated {
            airtable_id: record.id,
            row: ProductInsert {
                name: record.fields.name,
                price: record.fields.price.map(dollars_to_cents),
                liquidation: record.fields.liquidation.unwrap_or(false),
                visible_on_site: record.fields.visible_on_site.unwrap_or(false),
            },
            product_types: record.fields.product_types.unwrap(),
            image_front: record.fields.image_front.and_then(|imgs| {
                imgs.first().map(|img| ProductImage {
                    url: img.url.clone(),
                    filename: img.filename.clone(),
                })
            }),
            image_back: record.fields.image_back.and_then(|imgs| {
                imgs.first().map(|img| ProductImage {
                    url: img.url.clone(),
                    filename: img.filename.clone(),
                })
            }),
        }
    }
}

/// Load product records from Airtable JSON export
pub async fn load_products_from_export(
    data: AirtableRecords<ProductFields>,
) -> Result<Vec<ProductRowWithRelated>> {
    let mut rows = Vec::new();
    for (idx, record) in data.records.into_iter().enumerate() {
        validate_product_fields(&record.fields).with_context(|| {
            format!("Invalid product data in record {} (id: {})", idx, record.id)
        })?;

        rows.push(ProductRowWithRelated::from(record));
    }

    println!("Loaded {} product records from JSON", rows.len());
    Ok(rows)
}

fn validate_product_fields(fields: &ProductFields) -> Result<()> {
    if fields.name.trim().is_empty() {
        anyhow::bail!("product name cannot be empty");
    }
    if let Some(price) = fields.price {
        if price < 0.0 {
            anyhow::bail!("product price cannot be negative");
        }
    }
    Ok(())
}

/// Facture fields from Airtable
#[derive(Debug, Deserialize)]
pub struct FactureFields {
    #[serde(rename = "Numéro de facture")]
    facture_number: i64,
    #[serde(rename = "Client")]
    client: Vec<String>, // Airtable linked field (required)
    #[serde(rename = "Type")]
    facture_type: Option<String>,
    #[serde(rename = "Date")]
    date: Option<String>,
    #[serde(rename = "Événements")]
    event: Option<Vec<String>>, // Airtable linked field (optional)
    #[serde(rename = "Total fixe")]
    fixed_total: Option<f64>, // Will convert to cents
    #[serde(rename = "Annulée")]
    cancelled: Option<bool>, // Default to false
    #[serde(rename = "Réf. Ancienne")]
    paper_ref: Option<String>,
}

/// Migration-specific: Facture with unresolved foreign keys
/// This struct holds airtable IDs that need to be resolved before insertion
#[derive(Debug)]
pub struct MigrationFactureInsertWithFKs {
    pub row: MigrationFactureInsert,
    pub airtable_id: String,
    pub client_airtable_id: String,        // Required FK to resolve
    pub event_airtable_id: Option<String>, // Optional FK to resolve
}

impl From<AirtableRecord<FactureFields>> for MigrationFactureInsertWithFKs {
    fn from(record: AirtableRecord<FactureFields>) -> Self {
        // Extract client airtable ID (required - take first element)
        let client_airtable_id = record.fields.client.first().cloned().unwrap();

        let facture_type = match record.fields.facture_type.as_deref() {
            Some("Produits") => "Product",
            Some("Altération") => "Alteration",
            Some("Location") => "Location",
            None | Some(_) => "Product",
        };

        // Extract event airtable ID if present (optional - take first element)
        let event_airtable_id = record
            .fields
            .event
            .and_then(|events| events.first().cloned());

        MigrationFactureInsertWithFKs {
            airtable_id: record.id,
            client_airtable_id,
            event_airtable_id,
            row: MigrationFactureInsert {
                id: record.fields.facture_number,
                client_id: 0, // Will be resolved from airtable_mapping
                facture_type: Some(String::from(facture_type)),
                date: record.fields.date,
                event_id: None, // Will be resolved from airtable_mapping
                fixed_total: record.fields.fixed_total.map(dollars_to_cents),
                cancelled: record.fields.cancelled.unwrap_or(false),
                paper_ref: record.fields.paper_ref,
            },
        }
    }
}

/// Load facture records from Airtable JSON export
async fn load_factures_from_export(
    data: AirtableRecords<FactureFields>,
) -> Result<Vec<MigrationFactureInsertWithFKs>> {
    let mut rows = Vec::new();
    for (idx, record) in data.records.into_iter().enumerate() {
        validate_facture_fields(&record.fields).with_context(|| {
            format!("Invalid facture data in record {} (id: {})", idx, record.id)
        })?;

        rows.push(MigrationFactureInsertWithFKs::from(record));
    }

    println!("Loaded {} facture records from JSON", rows.len());
    Ok(rows)
}

fn validate_facture_fields(fields: &FactureFields) -> Result<()> {
    // Client is required
    if fields.client.is_empty() {
        anyhow::bail!("facture must have a client");
    }

    // Validate facture type if present
    if let Some(ref facture_type) = fields.facture_type {
        const VALID_TYPES: &[&str] = &["Produits", "Location", "Altération"];
        if !VALID_TYPES.contains(&facture_type.as_str()) {
            anyhow::bail!(
                "Invalid facture type: '{}'. Must be one of: {:?}",
                facture_type,
                VALID_TYPES
            );
        }
    }

    // Validate fixed_total if present
    if let Some(total) = fields.fixed_total {
        if total < 0.0 {
            anyhow::bail!("facture fixed_total cannot be negative");
        }
    }

    Ok(())
}

/// Load and insert factures with foreign key resolution
pub async fn load_and_insert_factures(
    pool: &SqlitePool,
    data: AirtableRecords<FactureFields>,
) -> Result<()> {
    let factures = load_factures_from_export(data).await?;

    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    for mut facture in factures {
        // Resolve client_id (required - will abort if not found)
        let client_id = resolve_airtable_id(pool, Table::Clients, &facture.client_airtable_id)
            .await
            .with_context(|| {
                format!(
                    "Failed to resolve client for facture (airtable_id: {})",
                    facture.airtable_id
                )
            })?;
        facture.row.client_id = client_id;

        // Resolve event_id (optional - if can't resolve, just set to None)
        if let Some(ref event_airtable_id) = facture.event_airtable_id {
            match resolve_airtable_id(pool, Table::Events, event_airtable_id).await {
                Ok(event_id) => {
                    facture.row.event_id = Some(event_id);
                }
                Err(_) => {
                    // Event not found - this is okay for optional FKs
                    println!(
                        "Warning: Event '{}' not found for facture '{}', setting to None",
                        event_airtable_id, facture.airtable_id
                    );
                    facture.row.event_id = None;
                }
            }
        }

        // Insert facture and get db_id
        let facture_id = facture
            .row
            .insert_one(&mut tx)
            .await?
            .context("Facture insertion must return an id")?;

        // Insert airtable mapping
        insert_airtable_id(&mut tx, Table::Factures, facture_id, facture.airtable_id).await?;
    }

    tx.commit().await.context("Failed to commit transaction")?;

    Ok(())
}

/// Facture item fields from Airtable
#[derive(Debug, Deserialize)]
pub struct FactureItemFields {
    #[serde(rename = "_itemType")]
    item_type: String, // "Product", "Location", or "Alteration"
    #[serde(rename = "Produit")]
    product: Vec<String>, // Airtable linked field (required)
    #[serde(rename = "Facture")]
    facture: Vec<String>, // Airtable linked field (required)
    #[serde(rename = "Prix")]
    price: Option<f64>, // Will convert to cents
    #[serde(rename = "Notes")]
    notes: Option<String>,
    #[serde(rename = "Quantité")]
    quantity: Option<i64>,

    // Produit-specific
    #[serde(rename = "Total extra taille forte")]
    extra_large_size: Option<f64>, // Will convert to cents
    #[serde(rename = "Rabais")]
    rebate_percent: Option<i64>,
    #[serde(rename = "Grandeur")]
    size: Option<String>,
    #[serde(rename = "Buste")]
    chest: Option<i64>,
    #[serde(rename = "Taille")]
    waist: Option<i64>,
    #[serde(rename = "Hanche")]
    hips: Option<i64>,
    #[serde(rename = "Couleur")]
    color: Option<String>,
    #[serde(rename = "Bénéficiaire")]
    beneficiary: Option<String>,
    #[serde(rename = "Item plancher")]
    floor_item: Option<bool>,

    // Location-specific
    #[serde(rename = "Assurances")]
    insurance: Option<f64>, // Will convert to cents
    #[serde(rename = "Autre frais")]
    other_costs: Option<f64>, // Will convert to cents

    // Alteration-specific
    #[serde(rename = "Rabais dollar")]
    rebate_dollar: Option<f64>, // Will convert to cents
}

/// Migration-specific: Facture item with unresolved foreign keys
#[derive(Debug)]
pub struct FactureItemRowWithFKs {
    pub row: FactureItemInsert,
    pub airtable_id: String,
    pub facture_airtable_id: String, // Required FK to resolve
    pub product_airtable_id: String, // Required FK to resolve
}

impl From<AirtableRecord<FactureItemFields>> for FactureItemRowWithFKs {
    fn from(record: AirtableRecord<FactureItemFields>) -> Self {
        // Extract facture airtable ID (required - take first element)
        let facture_airtable_id = record.fields.facture.first().cloned().unwrap();

        // Extract product airtable ID (required - take first element)
        let product_airtable_id = record.fields.product.first().cloned().unwrap();

        // Map item type from French to database values
        let item_type = match record.fields.item_type.as_str() {
            "Products" => "Product",
            "Location" => "Location",
            "Alteration" => "Alteration",
            other => other, // Keep as-is if already in database format
        };

        FactureItemRowWithFKs {
            airtable_id: record.id,
            facture_airtable_id,
            product_airtable_id,
            row: FactureItemInsert {
                facture_id: 0, // Will be resolved from airtable_mapping
                product_id: 0, // Will be resolved from airtable_mapping
                item_type: item_type.to_string(),
                price: record.fields.price.map(dollars_to_cents),
                notes: record.fields.notes,
                quantity: record.fields.quantity.unwrap_or(1),
                extra_large_size: record.fields.extra_large_size.map(dollars_to_cents),
                rebate_percent: record.fields.rebate_percent,
                size: record.fields.size,
                chest: record.fields.chest,
                waist: record.fields.waist,
                hips: record.fields.hips,
                color: record.fields.color,
                beneficiary: record.fields.beneficiary,
                floor_item: record.fields.floor_item.unwrap_or(false),
                insurance: record.fields.insurance.map(dollars_to_cents),
                other_costs: record.fields.other_costs.map(dollars_to_cents),
                rebate_dollar: record.fields.rebate_dollar.map(dollars_to_cents),
            },
        }
    }
}

/// Load facture_items records from Airtable JSON export
async fn load_facture_items_from_export(
    data: AirtableRecords<FactureItemFields>,
) -> Result<Vec<FactureItemRowWithFKs>> {
    let mut rows = Vec::new();
    for (idx, record) in data.records.into_iter().enumerate() {
        validate_facture_item_fields(&record.fields).with_context(|| {
            format!(
                "Invalid facture_item data in record {} (id: {})",
                idx, record.id
            )
        })?;

        rows.push(FactureItemRowWithFKs::from(record));
    }

    println!("Loaded {} facture_item records from JSON", rows.len());
    Ok(rows)
}

fn validate_facture_item_fields(fields: &FactureItemFields) -> Result<()> {
    // Facture is required
    if fields.facture.is_empty() {
        anyhow::bail!("facture_item must have a facture");
    }

    // Product is required
    if fields.product.is_empty() {
        anyhow::bail!("facture_item must have a product");
    }

    // Validate item type
    const VALID_TYPES: &[&str] = &["Products", "Location", "Alteration", "Produit"];
    if !VALID_TYPES.contains(&fields.item_type.as_str()) {
        anyhow::bail!(
            "Invalid facture_item type: '{}'. Must be one of: {:?}",
            fields.item_type,
            VALID_TYPES
        );
    }

    // Validate quantity if present
    if let Some(qty) = fields.quantity {
        if qty < 0 {
            anyhow::bail!("facture_item quantity cannot be negative");
        }
    }

    Ok(())
}

/// Load and insert facture_items with foreign key resolution
pub async fn load_and_insert_facture_items(
    pool: &SqlitePool,
    data: AirtableRecords<FactureItemFields>,
) -> Result<()> {
    let facture_items = load_facture_items_from_export(data).await?;

    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    for mut item in facture_items {
        // Resolve facture_id (required - will abort if not found)
        let facture_id = resolve_airtable_id(pool, Table::Factures, &item.facture_airtable_id)
            .await
            .with_context(|| {
                format!(
                    "Failed to resolve facture for facture_item (airtable_id: {})",
                    item.airtable_id
                )
            })?;
        item.row.facture_id = facture_id;

        // Resolve product_id (required - will abort if not found)
        let product_id = resolve_airtable_id(pool, Table::Products, &item.product_airtable_id)
            .await
            .with_context(|| {
                format!(
                    "Failed to resolve product for facture_item (airtable_id: {})",
                    item.airtable_id
                )
            })?;
        item.row.product_id = product_id;

        // Insert facture_item and get db_id
        let item_id = item
            .row
            .insert_one(&mut tx)
            .await?
            .context("Facture_item insertion must return an id")?;

        // Insert airtable mapping
        insert_airtable_id(&mut tx, Table::FactureItems, item_id, item.airtable_id).await?;
    }

    tx.commit().await.context("Failed to commit transaction")?;
    Ok(())
}

/// Payment fields from Airtable
#[derive(Debug, Deserialize)]
pub struct PaymentFields {
    #[serde(rename = "Facture")]
    facture: Vec<String>, // Airtable linked field (required)
    #[serde(rename = "Montant")]
    amount: f64, // Will convert to cents
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "Type")]
    payment_type: String,
    #[serde(rename = "Numéro de chèque")]
    cheque_number: Option<String>,
}

/// Migration-specific: Payment with unresolved foreign keys
#[derive(Debug)]
pub struct PaymentRowWithFKs {
    pub row: PaymentInsert,
    pub airtable_id: String,
    pub facture_airtable_id: String, // Required FK to resolve
}

impl From<AirtableRecord<PaymentFields>> for PaymentRowWithFKs {
    fn from(record: AirtableRecord<PaymentFields>) -> Self {
        // Extract facture airtable ID (required - take first element)
        let facture_airtable_id = record.fields.facture.first().cloned().unwrap();

        PaymentRowWithFKs {
            airtable_id: record.id,
            facture_airtable_id,
            row: PaymentInsert {
                facture_id: 0, // Will be resolved from airtable_mapping
                amount: dollars_to_cents(record.fields.amount),
                date: record.fields.date,
                payment_type: record.fields.payment_type,
                cheque_number: record.fields.cheque_number,
            },
        }
    }
}

/// Load payment records from Airtable JSON export
async fn load_payments_from_export(
    data: AirtableRecords<PaymentFields>,
) -> Result<Vec<PaymentRowWithFKs>> {
    let mut rows = Vec::new();
    for (idx, record) in data.records.into_iter().enumerate() {
        validate_payment_fields(&record.fields).with_context(|| {
            format!("Invalid payment data in record {} (id: {})", idx, record.id)
        })?;

        rows.push(PaymentRowWithFKs::from(record));
    }

    println!("Loaded {} payment records from JSON", rows.len());
    Ok(rows)
}

fn validate_payment_fields(fields: &PaymentFields) -> Result<()> {
    // Facture is required
    if fields.facture.is_empty() {
        anyhow::bail!("payment must have a facture");
    }

    // Validate amount
    if fields.amount < 0.0 {
        anyhow::bail!("payment amount cannot be negative");
    }

    // Validate date
    if fields.date.trim().is_empty() {
        anyhow::bail!("payment date cannot be empty");
    }

    // Validate payment type
    if fields.payment_type.trim().is_empty() {
        anyhow::bail!("payment type cannot be empty");
    }

    Ok(())
}

/// Load and insert payments with foreign key resolution
pub async fn load_and_insert_payments(
    pool: &SqlitePool,
    data: AirtableRecords<PaymentFields>,
) -> Result<()> {
    let payments = load_payments_from_export(data).await?;

    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    for mut payment in payments {
        // Resolve facture_id (required - will abort if not found)
        let facture_id = resolve_airtable_id(pool, Table::Factures, &payment.facture_airtable_id)
            .await
            .with_context(|| {
                format!(
                    "Failed to resolve facture for payment (airtable_id: {})",
                    payment.airtable_id
                )
            })?;
        payment.row.facture_id = facture_id;

        // Insert payment and get db_id
        let payment_id = payment
            .row
            .insert_one(&mut tx)
            .await?
            .context("Payment insertion must return an id")?;

        // Insert airtable mapping
        insert_airtable_id(&mut tx, Table::Payments, payment_id, payment.airtable_id).await?;
    }

    tx.commit().await.context("Failed to commit transaction")?;
    Ok(())
}

/// Refund fields from Airtable
#[derive(Debug, Deserialize)]
pub struct RefundFields {
    #[serde(rename = "Facture")]
    facture: Vec<String>, // Airtable linked field (required)
    #[serde(rename = "Montant")]
    amount: f64, // Will convert to cents
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "Type")]
    refund_type: String,
    #[serde(rename = "Numéro de chèque")]
    cheque_number: Option<String>,
}

/// Migration-specific: Refund with unresolved foreign keys
#[derive(Debug)]
pub struct RefundRowWithFKs {
    pub row: RefundInsert,
    pub airtable_id: String,
    pub facture_airtable_id: String, // Required FK to resolve
}

impl From<AirtableRecord<RefundFields>> for RefundRowWithFKs {
    fn from(record: AirtableRecord<RefundFields>) -> Self {
        // Extract facture airtable ID (required - take first element)
        let facture_airtable_id = record.fields.facture.first().cloned().unwrap();

        RefundRowWithFKs {
            airtable_id: record.id,
            facture_airtable_id,
            row: RefundInsert {
                facture_id: 0, // Will be resolved from airtable_mapping
                amount: dollars_to_cents(record.fields.amount),
                date: record.fields.date,
                refund_type: record.fields.refund_type,
                cheque_number: record.fields.cheque_number,
            },
        }
    }
}

/// Load refund records from Airtable JSON export
async fn load_refunds_from_export(
    data: AirtableRecords<RefundFields>,
) -> Result<Vec<RefundRowWithFKs>> {
    let mut rows = Vec::new();
    for (idx, record) in data.records.into_iter().enumerate() {
        validate_refund_fields(&record.fields).with_context(|| {
            format!("Invalid refund data in record {} (id: {})", idx, record.id)
        })?;

        rows.push(RefundRowWithFKs::from(record));
    }

    println!("Loaded {} refund records from JSON", rows.len());
    Ok(rows)
}

fn validate_refund_fields(fields: &RefundFields) -> Result<()> {
    // Facture is required
    if fields.facture.is_empty() {
        anyhow::bail!("refund must have a facture");
    }

    // Validate amount
    if fields.amount < 0.0 {
        anyhow::bail!("refund amount cannot be negative");
    }

    // Validate date
    if fields.date.trim().is_empty() {
        anyhow::bail!("refund date cannot be empty");
    }

    // Validate refund type
    if fields.refund_type.trim().is_empty() {
        anyhow::bail!("refund type cannot be empty");
    }

    Ok(())
}

/// Load and insert refunds with foreign key resolution
pub async fn load_and_insert_refunds(
    pool: &SqlitePool,
    data: AirtableRecords<RefundFields>,
) -> Result<()> {
    let refunds = load_refunds_from_export(data).await?;

    let mut tx = pool.begin().await.context("Failed to begin transaction")?;
    for mut refund in refunds {
        // Resolve facture_id (required - will abort if not found)
        let facture_id = resolve_airtable_id(pool, Table::Factures, &refund.facture_airtable_id)
            .await
            .with_context(|| {
                format!(
                    "Failed to resolve facture for refund (airtable_id: {})",
                    refund.airtable_id
                )
            })?;
        refund.row.facture_id = facture_id;

        // Insert refund and get db_id
        let refund_id = refund
            .row
            .insert_one(&mut tx)
            .await?
            .context("Refund insertion must return an id")?;

        // Insert airtable mapping
        insert_airtable_id(&mut tx, Table::Refunds, refund_id, refund.airtable_id).await?;
    }

    tx.commit().await.context("Failed to commit transaction")?;
    Ok(())
}

/// Statut fields from Airtable
#[derive(Debug, Deserialize)]
pub struct StatutFields {
    #[serde(rename = "Facture")]
    facture: Vec<String>, // Airtable linked field (required)
    #[serde(rename = "Item de facture")]
    facture_item: Vec<String>, // Airtable linked field (required)
    #[serde(rename = "Type")]
    statut_type: String,
    #[serde(rename = "Date copy")]
    date: String, // ISO 8601 timestamp, need to extract date
    #[serde(rename = "Couturière")]
    seamstress: Option<String>,
}

/// Migration-specific: Statut with unresolved foreign keys
#[derive(Debug)]
pub struct StatutRowWithFKs {
    pub row: StatutInsert,
    pub airtable_id: String,
    pub facture_airtable_id: String,      // Required FK to resolve
    pub facture_item_airtable_id: String, // Required FK to resolve
}

/// Extract date from ISO 8601 timestamp (e.g., "2024-09-03T00:00:00Z" → "2024-09-03")
fn extract_date_from_timestamp(timestamp: &str) -> String {
    timestamp.split('T').next().unwrap_or(timestamp).to_string()
}

impl From<AirtableRecord<StatutFields>> for StatutRowWithFKs {
    fn from(record: AirtableRecord<StatutFields>) -> Self {
        // Extract facture airtable ID (required - take first element)
        let facture_airtable_id = record.fields.facture.first().cloned().unwrap();

        // Extract facture_item airtable ID (required - take first element)
        let facture_item_airtable_id = record.fields.facture_item.first().cloned().unwrap();

        StatutRowWithFKs {
            airtable_id: record.id,
            facture_airtable_id,
            facture_item_airtable_id,
            row: StatutInsert {
                facture_id: 0,      // Will be resolved from airtable_mapping
                facture_item_id: 0, // Will be resolved from airtable_mapping
                statut_type: record.fields.statut_type,
                date: extract_date_from_timestamp(&record.fields.date),
                seamstress: record.fields.seamstress,
            },
        }
    }
}

/// Load statut records from Airtable JSON export
async fn load_statuts_from_export(
    data: AirtableRecords<StatutFields>,
) -> Result<Vec<StatutRowWithFKs>> {
    let mut rows = Vec::new();
    for (idx, record) in data.records.into_iter().enumerate() {
        validate_statut_fields(&record.fields).with_context(|| {
            format!("Invalid statut data in record {} (id: {})", idx, record.id)
        })?;

        rows.push(StatutRowWithFKs::from(record));
    }

    println!("Loaded {} statut records from JSON", rows.len());
    Ok(rows)
}

fn validate_statut_fields(fields: &StatutFields) -> Result<()> {
    // Facture is required
    if fields.facture.is_empty() {
        anyhow::bail!("statut must have a facture");
    }

    // Facture item is required
    if fields.facture_item.is_empty() {
        anyhow::bail!("statut must have a facture_item");
    }

    // Validate statut type
    if fields.statut_type.trim().is_empty() {
        anyhow::bail!("statut type cannot be empty");
    }

    // Validate date
    if fields.date.trim().is_empty() {
        anyhow::bail!("statut date cannot be empty");
    }

    Ok(())
}

/// Load and insert statuts with foreign key resolution
pub async fn load_and_insert_statuts(
    pool: &SqlitePool,
    data: AirtableRecords<StatutFields>,
) -> Result<()> {
    let statuts = load_statuts_from_export(data).await?;

    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    for mut statut in statuts {
        // Resolve facture_id (required - will abort if not found)
        let facture_id = resolve_airtable_id(pool, Table::Factures, &statut.facture_airtable_id)
            .await
            .with_context(|| {
                format!(
                    "Failed to resolve facture for statut (airtable_id: {})",
                    statut.airtable_id
                )
            })?;
        statut.row.facture_id = facture_id;

        // Resolve facture_item_id (required - will abort if not found)
        let facture_item_id =
            resolve_airtable_id(pool, Table::FactureItems, &statut.facture_item_airtable_id)
                .await
                .with_context(|| {
                    format!(
                        "Failed to resolve facture_item for statut (airtable_id: {})",
                        statut.airtable_id
                    )
                })?;
        statut.row.facture_item_id = facture_item_id;

        // Insert statut and get db_id
        let statut_id = statut
            .row
            .insert_one(&mut tx)
            .await?
            .context("Statut insertion must return an id")?;

        // Insert airtable mapping
        insert_airtable_id(&mut tx, Table::Statuts, statut_id, statut.airtable_id).await?;
    }

    tx.commit().await.context("Failed to commit transaction")?;
    Ok(())
}

pub async fn check_counts(pool: &SqlitePool) -> Result<()> {
    let tables = vec![
        Table::Clients,
        Table::Config,
        Table::Events,
        Table::ProductTypes,
        Table::Products,
        Table::Factures,
        Table::FactureItems,
        Table::Payments,
        Table::Refunds,
        Table::Statuts,
    ];
    for t in tables {
        count_check(pool, t).await?;
    }
    Ok(())
}

pub async fn load_records<R, T>(pool: &SqlitePool, data: AirtableRecords<R>) -> Result<()>
where
    WithId<T>: From<AirtableRecord<R>>,
    T: HasTable,
    T: Insertable,
{
    let mut converted = Vec::new();
    for r in data.records {
        converted.push(WithId::<T>::from(r));
    }

    let mut tx: sqlx::Transaction<'_, sqlx::Sqlite> =
        pool.begin().await.context("Failed to begin transaction")?;
    for with_id in converted {
        let maybe_id = with_id.row.insert_one(&mut tx).await?;
        if let Some(id) = maybe_id {
            insert_airtable_id(&mut tx, T::table(), id, with_id.airtable_id).await?;
        }
    }
    tx.commit().await.context("Failed to commit transaction")?;

    Ok(())
}

async fn count_check(pool: &SqlitePool, table_name: Table) -> Result<()> {
    let result: (i64,) = sqlx::query_as(&format!(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='{}'",
        table_name
    ))
    .fetch_one(pool)
    .await
    .context("Failed to verify config table")?;

    if result.0 == 0 {
        anyhow::bail!(format!(
            "Table {} is does not exists, please run the migrations.",
            table_name
        ))
    }

    let query = format!("SELECT COUNT(*) FROM {}", table_name);
    let (count,): (i64,) = sqlx::query_as(&query).fetch_one(pool).await?;

    if count > 0 {
        anyhow::bail!(format!(
            "Table {} is not empty. This `load` cli only works on empty databases.",
            table_name
        ))
    }

    Ok(())
}

async fn insert_airtable_id(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    table: Table,
    id: i64,
    airtable_id: String,
) -> Result<()> {
    // Insert mapping into airtable_mapping table
    sqlx::query(&format!(
        "INSERT INTO airtable_mapping (table_name, airtable_id, db_id)
             VALUES ('{}', ?, ?)",
        table.table_name()
    ))
    .bind(&airtable_id)
    .bind(id)
    .execute(&mut **tx)
    .await
    .with_context(|| {
        format!(
            "Failed to insert airtable mapping for {}: id = {}, airtable_id = {}",
            table, id, airtable_id
        )
    })?;
    Ok(())
}

/// Resolve an airtable_id to a database ID using the airtable_mapping table
/// Returns an error if the mapping is not found
async fn resolve_airtable_id(
    pool: &SqlitePool,
    table_name: Table,
    airtable_id: &str,
) -> Result<i64> {
    let result: Option<(i64,)> = sqlx::query_as(&format!(
        "SELECT db_id FROM airtable_mapping WHERE table_name = '{}' AND airtable_id = ?",
        table_name
    ))
    .bind(airtable_id)
    .fetch_optional(pool)
    .await
    .with_context(|| {
        format!(
            "Failed to query airtable_mapping for table={}, airtable_id={}",
            table_name, airtable_id
        )
    })?;

    match result {
        Some((db_id,)) => Ok(db_id),
        None => anyhow::bail!(
            "Cannot resolve airtable_id '{}' in table '{}'. The referenced record must be imported first.",
            airtable_id,
            table_name
        ),
    }
}

/// Insert a product with all its related data (types and images)
async fn insert_product_with_related(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    product: ProductRowWithRelated,
) -> Result<()> {
    // 1. Insert product row and get database ID
    let product_id = product
        .row
        .insert_one(tx)
        .await?
        .context("Product insertion must return an id")?;

    // 2. Insert airtable mapping
    insert_airtable_id(tx, Table::Products, product_id, product.airtable_id).await?;

    // 3. Insert product_product_types records
    for product_type_name in product.product_types {
        sqlx::query(
            "INSERT INTO product_product_types (product_id, product_type_name)
             VALUES (?, ?)",
        )
        .bind(product_id)
        .bind(&product_type_name)
        .execute(&mut **tx)
        .await
        .with_context(|| {
            format!(
                "Failed to insert product_product_type for product_id={}, type={}",
                product_id, product_type_name
            )
        })?;
    }

    // 4. Insert front image if present (use product's created_at timestamp)
    if let Some(image) = product.image_front {
        let image_row = ProductImageInsert {
            product_id,
            url: image.url,
            filename: image.filename,
            position: "front".to_string(),
        };
        image_row.insert_one(tx).await?;
    }

    // 5. Insert back image if present (use product's created_at timestamp)
    if let Some(image) = product.image_back {
        let image_row = ProductImageInsert {
            product_id,
            url: image.url,
            filename: image.filename,
            position: "back".to_string(),
        };
        image_row.insert_one(tx).await?;
    }

    Ok(())
}

/// Load and insert products with related data
pub async fn load_and_insert_products(
    pool: &SqlitePool,
    data: AirtableRecords<ProductFields>,
) -> Result<()> {
    let products = load_products_from_export(data).await?;

    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    for product in products {
        insert_product_with_related(&mut tx, product).await?;
    }

    tx.commit().await.context("Failed to commit transaction")?;
    Ok(())
}
