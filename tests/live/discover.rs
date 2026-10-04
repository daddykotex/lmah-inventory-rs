use anyhow::{Context, Result, bail};
use reqwest::Client;
use scraper::{Html, Selector};
use url::Url;

use super::session::AuthedSession;

/// Scrape the `/factures` listing and return the first facture id, or `None`
/// if the listing is empty. Keeps tests data-agnostic.
pub async fn first_facture_id(
    client: &Client,
    session: &AuthedSession,
    base: &Url,
) -> Result<Option<String>> {
    let url = base.join("/factures").context("joining /factures url")?;
    let resp = session
        .apply(client.get(url))
        .send()
        .await
        .context("GET /factures")?;
    if !resp.status().is_success() {
        bail!("/factures returned {}", resp.status());
    }
    let body = resp.text().await.context("reading /factures body")?;
    Ok(extract_first_facture_id(&body))
}

fn extract_first_facture_id(html: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    let selector = Selector::parse("a[href*=\"/factures/\"]").ok()?;
    for el in doc.select(&selector) {
        let Some(href) = el.value().attr("href") else {
            continue;
        };
        if let Some(id) = parse_facture_items_href(href) {
            return Some(id);
        }
    }
    None
}

fn parse_facture_items_href(href: &str) -> Option<String> {
    let path = href.split('?').next()?;
    let rest = path.strip_prefix("/factures/")?;
    let (id, tail) = rest.split_once('/')?;
    if tail != "items" || id.is_empty() {
        return None;
    }
    Some(id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_items_href() {
        assert_eq!(
            parse_facture_items_href("/factures/abc123/items").as_deref(),
            Some("abc123")
        );
        assert_eq!(
            parse_facture_items_href("/factures/abc/items?x=1").as_deref(),
            Some("abc")
        );
        assert_eq!(parse_facture_items_href("/factures/abc"), None);
        assert_eq!(parse_facture_items_href("/factures/abc/items/42"), None);
    }

    #[test]
    fn finds_first_id_in_listing() {
        let html = r#"
          <html><body>
            <a href="/factures/new">New</a>
            <a href="/factures/abc-1/items">Row</a>
            <a href="/factures/xyz-2/items">Another</a>
          </body></html>
        "#;
        assert_eq!(extract_first_facture_id(html).as_deref(), Some("abc-1"));
    }
}
