use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::cli::migration::AirtableRecords;

/// Thin wrapper over `reqwest::Client` for Airtable REST API calls.
///
/// Handles bearer-token auth and offset-based pagination. Table names are
/// appended to `base_url` and URL-encoded (names contain spaces and accents,
/// e.g. `Items de factures`, `Événements`).
#[derive(Debug, Clone)]
pub struct AirtableClient {
    http: Client,
    base_url: String,
    pat: String,
}

#[derive(Debug, Deserialize)]
struct Page {
    #[serde(default)]
    records: Vec<Value>,
    #[serde(default)]
    offset: Option<String>,
}

impl AirtableClient {
    pub fn new(base_url: String, pat: String) -> Self {
        let base_url = if base_url.ends_with('/') {
            base_url
        } else {
            format!("{}/", base_url)
        };
        Self {
            http: Client::new(),
            base_url,
            pat,
        }
    }

    fn table_url(&self, table: &str) -> String {
        format!("{}{}", self.base_url, urlencode(table))
    }

    async fn fetch_pages(&self, table: &str, view: Option<&str>) -> Result<Vec<Value>> {
        let url = self.table_url(table);
        let mut all: Vec<Value> = Vec::new();
        let mut offset: Option<String> = None;

        loop {
            let mut query: Vec<(&str, String)> = Vec::new();
            query.push(("pageSize", "100".to_string()));
            if let Some(v) = view {
                query.push(("view", v.to_string()));
            }
            if let Some(o) = &offset {
                query.push(("offset", o.clone()));
            }

            let resp = self
                .http
                .get(&url)
                .bearer_auth(&self.pat)
                .query(&query)
                .send()
                .await
                .with_context(|| format!("GET {} failed", url))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("Airtable {} returned {}: {}", table, status, body);
            }

            let page: Page = resp
                .json()
                .await
                .with_context(|| format!("Decoding Airtable response for table {}", table))?;

            all.extend(page.records);

            match page.offset {
                Some(next) => offset = Some(next),
                None => break,
            }
        }

        Ok(all)
    }

    /// Fetch every record in `table`, decoding each `fields` object into `T`.
    pub async fn fetch_all<T>(&self, table: &str) -> Result<AirtableRecords<T>>
    where
        T: DeserializeOwned,
    {
        let records = self.fetch_pages(table, None).await?;
        let typed: AirtableRecords<T> = serde_json::from_value(serde_json::json!({
            "records": records,
        }))
        .with_context(|| format!("Decoding records for table {}", table))?;
        Ok(typed)
    }

    /// Fetch every record in `table` as raw JSON values, preserving all fields.
    /// Used when we need to mutate fields before decoding (product URL rewrite,
    /// facture_item `_itemType` injection).
    pub async fn fetch_all_raw(&self, table: &str) -> Result<Vec<Value>> {
        self.fetch_pages(table, None).await
    }

    /// Fetch record ids from a specific view, preserving the view's sort order.
    pub async fn fetch_view_ids(&self, table: &str, view: &str) -> Result<Vec<String>> {
        let records = self.fetch_pages(table, Some(view)).await?;
        let mut ids = Vec::with_capacity(records.len());
        for r in records {
            let id = r
                .get("id")
                .and_then(|v| v.as_str())
                .with_context(|| format!("Record in view {} missing id", view))?
                .to_string();
            ids.push(id);
        }
        Ok(ids)
    }
}

fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        let unreserved =
            b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~';
        if unreserved {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{:02X}", b));
        }
    }
    out
}
