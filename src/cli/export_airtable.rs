use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::cli::airtable_client::AirtableClient;
use crate::cli::migration::{
    AirtableExport, AirtableRecords, ClientFields, ConfigFields, EventFields, FactureFields,
    FactureItemFields, PaymentFields, ProductFields, ProductTypeFields, ProductViewOrderingFields,
    RefundFields, StatutFields, check_counts, load_and_insert_facture_items,
    load_and_insert_factures, load_and_insert_payments, load_and_insert_product_view_orderings,
    load_and_insert_products, load_and_insert_refunds, load_and_insert_statuts, load_records,
    sort_export_by_created_time,
};
use crate::server::database::connect_to_url;
use crate::server::models::clients::ClientInsert;
use crate::server::models::config::ConfigInsert;
use crate::server::models::events::EventInsert;
use crate::server::models::product_types::ProductTypeRow;

/// Public bucket name the Scala exporter rewrote attachment URLs against.
/// Overridable via `LMAH_GOOGLE_PUBLIC_BUCKET_NAME` so the result matches
/// whatever `data/valid-export.db` holds.
const DEFAULT_PUBLIC_BUCKET: &str = "lmah-site-public-images";

const PRODUCT_TYPES: &[&str] = &[
    "Robe de mariée",
    "Robe de bal",
    "Robe de bouquetière",
    "Robe de mère de la mariée",
    "Ensemble collier et boucle d'oreilles",
    "Bracelets",
    "Voile",
    "Crinoline",
    "Broches à cheveux",
    "Ceinturon",
    "Epinglette",
    "Diademe",
    "Jarretière",
    "Accessoire",
    "Altération",
    "Gaine",
    "Location",
    "Soya et Koala",
];

const PRODUCT_VIEWS: &[(&str, &str)] = &[
    ("robes_de_mariees", "Page de robes de mariée"),
    ("robes_de_bal", "Page de robes de bal"),
    ("robes_de_meres", "Page de robes de mères de mariée"),
];

pub struct ExportAirtableOptions {
    pub target: String,
    pub airtable_pat: String,
    pub airtable_url: String,
    pub public_bucket: Option<String>,
}

pub async fn run(opts: ExportAirtableOptions) -> Result<()> {
    let bucket = opts
        .public_bucket
        .unwrap_or_else(|| DEFAULT_PUBLIC_BUCKET.to_string());

    println!("LMAH Inventory - Airtable Exporter");
    println!("===================================");
    println!("Target: {}", opts.target);
    println!("Airtable: {}", opts.airtable_url);
    println!("Public bucket (for image URL rewrite): {}", bucket);
    println!();

    let client = AirtableClient::new(opts.airtable_url, opts.airtable_pat);

    println!("Connecting to database...");
    let pool = connect_to_url(&opts.target).await?;
    check_counts(&pool).await?;
    println!("✓ Database connection established and empty\n");

    // ===== FETCH ALL TABLES FROM AIRTABLE =====
    println!("Fetching tables from Airtable...");
    let config: AirtableRecords<ConfigFields> = client.fetch_all("Site").await?;
    println!("  Site               (config): {}", config.records.len());

    let clients: AirtableRecords<ClientFields> = client.fetch_all("Clients").await?;
    println!("  Clients                    : {}", clients.records.len());

    let events: AirtableRecords<EventFields> = client.fetch_all("Événements").await?;
    println!("  Événements          (events): {}", events.records.len());

    let products_raw = client.fetch_all_raw("Produits").await?;
    println!("  Produits          (products): {}", products_raw.len());

    let factures_raw = client.fetch_all_raw("Factures").await?;
    println!("  Factures                   : {}", factures_raw.len());

    let items_raw = client.fetch_all_raw("Items de factures").await?;
    println!("  Items de factures          : {}", items_raw.len());

    let payments: AirtableRecords<PaymentFields> = client.fetch_all("Paiements").await?;
    println!("  Paiements         (payments): {}", payments.records.len());

    let refunds: AirtableRecords<RefundFields> = client.fetch_all("Remboursements").await?;
    println!("  Remboursements     (refunds): {}", refunds.records.len());

    let statuts: AirtableRecords<StatutFields> = client.fetch_all("Statuts").await?;
    println!("  Statuts                    : {}", statuts.records.len());

    // ===== PRODUCTS: rewrite attachment URLs to GCS =====
    let products_rewritten: Vec<Value> = products_raw
        .into_iter()
        .map(|mut rec| {
            rewrite_product_attachments(&mut rec, &bucket);
            rec
        })
        .collect();
    let products: AirtableRecords<ProductFields> =
        serde_json::from_value(json!({ "records": products_rewritten }))
            .context("Decoding Produits records after URL rewrite")?;

    // ===== FACTURES: decode first so we can build the type map =====
    let factures: AirtableRecords<FactureFields> =
        serde_json::from_value(json!({ "records": &factures_raw }))
            .context("Decoding Factures records")?;

    // Build airtable_facture_id -> _itemType map for facture_items.
    // Values match what Scala's FactureType.toString produced:
    //   "Produits"   -> "Products"
    //   "Location"   -> "Location"
    //   "Altération" -> "Alteration"
    //   missing/other -> "Products"
    let facture_type_map = build_facture_type_map(&factures_raw);

    // ===== FACTURE ITEMS: inject _itemType from parent facture =====
    let items_with_type: Vec<Value> = items_raw
        .into_iter()
        .map(|mut rec| {
            inject_item_type(&mut rec, &facture_type_map);
            rec
        })
        .collect();
    let facture_items: AirtableRecords<FactureItemFields> =
        serde_json::from_value(json!({ "records": items_with_type }))
            .context("Decoding Items de factures after _itemType injection")?;

    // ===== PRODUCT TYPES (hardcoded 18) =====
    let product_types = build_hardcoded_product_types();

    // ===== PRODUCT VIEW ORDERINGS =====
    println!("\nFetching product view orderings...");
    let mut view_records: Vec<Value> = Vec::with_capacity(PRODUCT_VIEWS.len());
    for (slug, view_name) in PRODUCT_VIEWS {
        let ids = client.fetch_view_ids("Produits", view_name).await?;
        println!("  {}: {} products", view_name, ids.len());
        view_records.push(json!({
            "id": slug,
            "createdTime": "1970-01-01T00:00:00.000Z",
            "fields": { "productIds": ids },
        }));
    }
    let product_view_orderings: AirtableRecords<ProductViewOrderingFields> =
        serde_json::from_value(json!({ "records": view_records }))
            .context("Decoding product_view_orderings records")?;

    // ===== ASSEMBLE AND SORT =====
    let export = AirtableExport {
        config,
        clients,
        product_types,
        events,
        products,
        factures,
        facture_items,
        payments,
        refunds,
        statuts,
        product_view_orderings,
    };
    let export = sort_export_by_created_time(export);

    // ===== INSERT IN DEPENDENCY ORDER =====
    println!("\nStep: Inserting config records...");
    load_records::<ConfigFields, ConfigInsert>(&pool, export.config).await?;

    println!("Step: Inserting client records...");
    load_records::<ClientFields, ClientInsert>(&pool, export.clients).await?;

    println!("Step: Inserting product_types records...");
    load_records::<ProductTypeFields, ProductTypeRow>(&pool, export.product_types).await?;

    println!("Step: Inserting events records...");
    load_records::<EventFields, EventInsert>(&pool, export.events).await?;

    println!("Step: Inserting products with related data...");
    load_and_insert_products(&pool, export.products).await?;

    println!("Step: Inserting factures...");
    load_and_insert_factures(&pool, export.factures).await?;

    println!("Step: Inserting facture_items...");
    load_and_insert_facture_items(&pool, export.facture_items).await?;

    println!("Step: Inserting payments...");
    load_and_insert_payments(&pool, export.payments).await?;

    println!("Step: Inserting refunds...");
    load_and_insert_refunds(&pool, export.refunds).await?;

    println!("Step: Inserting statuts...");
    load_and_insert_statuts(&pool, export.statuts).await?;

    println!("Step: Inserting product view orderings...");
    load_and_insert_product_view_orderings(&pool, export.product_view_orderings).await?;

    println!("\n✓ Export complete");
    Ok(())
}

fn rewrite_product_attachments(record: &mut Value, bucket: &str) {
    let product_id = record
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let Some(fields) = record.get_mut("fields").and_then(|v| v.as_object_mut()) else {
        return;
    };
    for key in ["imageSrc", "imageSrc2"] {
        if let Some(Value::Array(attachments)) = fields.get_mut(key) {
            for att in attachments.iter_mut() {
                let Some(obj) = att.as_object_mut() else {
                    continue;
                };
                let att_id = obj
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let new_url = format!(
                    "https://storage.googleapis.com/{}/{}-{}.jpg",
                    bucket, product_id, att_id
                );
                obj.insert("url".to_string(), Value::String(new_url));
            }
        }
    }
}

fn build_facture_type_map(factures_raw: &[Value]) -> HashMap<String, String> {
    let mut map = HashMap::with_capacity(factures_raw.len());
    for rec in factures_raw {
        let Some(id) = rec.get("id").and_then(|v| v.as_str()) else {
            continue;
        };
        let airtable_type = rec
            .get("fields")
            .and_then(|f| f.get("Type"))
            .and_then(|v| v.as_str());
        let item_type = match airtable_type {
            Some("Produits") => "Products",
            Some("Location") => "Location",
            Some("Altération") => "Alteration",
            _ => "Products",
        };
        map.insert(id.to_string(), item_type.to_string());
    }
    map
}

fn inject_item_type(record: &mut Value, facture_type_map: &HashMap<String, String>) {
    let facture_id = record
        .get("fields")
        .and_then(|f| f.get("Facture"))
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let item_type = facture_id
        .as_deref()
        .and_then(|id| facture_type_map.get(id).cloned())
        .unwrap_or_else(|| "Products".to_string());

    if let Some(fields) = record.get_mut("fields").and_then(|v| v.as_object_mut()) {
        fields.insert("_itemType".to_string(), Value::String(item_type));
    }
}

fn build_hardcoded_product_types() -> AirtableRecords<ProductTypeFields> {
    let records: Vec<Value> = PRODUCT_TYPES
        .iter()
        .enumerate()
        .map(|(idx, name)| {
            json!({
                "id": format!("rec_hardcoded_pt_{}", idx + 1),
                "createdTime": "2019-02-13T04:56:46.000Z",
                "fields": { "Name": name },
            })
        })
        .collect();
    serde_json::from_value(json!({ "records": records }))
        .expect("hardcoded product_types JSON must decode")
}
