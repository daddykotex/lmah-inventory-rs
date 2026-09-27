use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;
use sqlx::SqlitePool;
use std::path::Path;

/// (ordering table, output file stem). The ordering table drives both
/// membership and display order (ORDER BY position ASC).
const OUTPUTS: &[(&str, &str)] = &[
    ("robes_de_mariees", "robes_de_mariees"),
    ("robes_de_bal", "robes_de_bals"),
    ("robes_de_meres", "robes_de_meres"),
];

/// One row returned by the SQL query below.
#[derive(Debug, sqlx::FromRow)]
struct ProductImageRow {
    product_id: i64,
    name: String,
    price: Option<i64>,
    liquidation: bool,
    image_id: Option<i64>,
    image_url: Option<String>,
}

/// Shape written to the JSON file. `prixWow` is `true` when liquidation, else
/// serialized as `null` (matches the Python script's output).
#[derive(Debug, Serialize)]
struct SiteEntry {
    id: String,
    name: String,
    price: i64,
    #[serde(rename = "reducedPrice")]
    reduced_price: i64,
    #[serde(rename = "prixWow")]
    prix_wow: Value,
    #[serde(rename = "imageSrc")]
    image_src: String,
    #[serde(rename = "imageId")]
    image_id: String,
}

pub struct SyncSiteOptions<'a> {
    pub db_url: &'a str,
    pub out_dir: &'a Path,
}

pub async fn run(opts: SyncSiteOptions<'_>) -> Result<()> {
    if !opts.out_dir.exists() {
        anyhow::bail!(
            "Output directory does not exist: {}",
            opts.out_dir.display()
        );
    }

    println!("LMAH Inventory - Site Sync");
    println!("==========================");
    println!("Database: {}", opts.db_url);
    println!("Output:   {}", opts.out_dir.display());
    println!();

    let pool = crate::server::database::connect_to_url(&opts.db_url.to_string()).await?;

    for (order_table, file_stem) in OUTPUTS {
        println!("→ {} → {}.json", order_table, file_stem);
        let rows = query_rows(&pool, order_table).await?;
        let entries = build_entries(rows);
        write_json(opts.out_dir, file_stem, &entries)?;
        println!("  wrote {} entries", entries.len());
    }

    Ok(())
}

async fn query_rows(pool: &SqlitePool, order_table: &str) -> Result<Vec<ProductImageRow>> {
    // The table name is a compile-time constant from OUTPUTS, so string
    // interpolation here is safe.
    let sql = format!(
        r#"
        SELECT p.id       AS product_id,
               p.name     AS name,
               p.price    AS price,
               p.liquidation AS liquidation,
               pi.id      AS image_id,
               pi.url     AS image_url
        FROM {order_table} o
        JOIN products p ON p.id = o.product_id
        LEFT JOIN product_images pi ON pi.product_id = p.id
        ORDER BY o.position ASC,
                 CASE pi.position WHEN 'front' THEN 0 WHEN 'back' THEN 1 ELSE 2 END
        "#
    );
    let rows: Vec<ProductImageRow> = sqlx::query_as(&sql)
        .fetch_all(pool)
        .await
        .with_context(|| format!("Failed to query products from {}", order_table))?;

    Ok(rows)
}

fn build_entries(rows: Vec<ProductImageRow>) -> Vec<SiteEntry> {
    let mut entries = Vec::with_capacity(rows.len());
    for row in rows {
        let (Some(image_id), Some(image_src)) = (row.image_id, row.image_url) else {
            eprintln!(
                "  skip product {} ({}): no image on file",
                row.product_id, row.name
            );
            continue;
        };
        let Some(price_cents) = row.price else {
            eprintln!(
                "  skip product {} ({}): price is null",
                row.product_id, row.name
            );
            continue;
        };

        entries.push(SiteEntry {
            id: row.product_id.to_string(),
            name: row.name,
            price: price_cents / 100,
            reduced_price: -1,
            prix_wow: if row.liquidation {
                Value::Bool(true)
            } else {
                Value::Null
            },
            image_src,
            image_id: image_id.to_string(),
        });
    }
    entries
}

fn write_json(out_dir: &Path, file_stem: &str, entries: &[SiteEntry]) -> Result<()> {
    let path = out_dir.join(format!("{}.json", file_stem));
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut buf = Vec::new();
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    entries
        .serialize(&mut ser)
        .context("Failed to serialize entries to JSON")?;
    std::fs::write(&path, &buf).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}
