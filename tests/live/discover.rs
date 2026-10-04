use anyhow::{Context, Result, bail};
use reqwest::Client;
use scraper::{Html, Selector};
use url::Url;

use super::session::AuthedSession;

/// Fetch a listing page and extract the first detail URL matching
/// `detail_pattern` (an id placeholder is written as `{id}`).
///
/// Keeps tests data-agnostic: we don't hardcode ids, and if the listing is
/// empty we return `None` so the caller can skip rather than fail.
pub async fn first_detail_path(
    client: &Client,
    session: &AuthedSession,
    base: &Url,
    list_path: &str,
    detail_pattern: &str,
) -> Result<Option<String>> {
    let url = base.join(list_path).context("joining listing url")?;
    let resp = session
        .apply(client.get(url))
        .send()
        .await
        .with_context(|| format!("GET {list_path}"))?;
    if !resp.status().is_success() {
        bail!("{list_path} returned {}", resp.status());
    }
    let body = resp
        .text()
        .await
        .with_context(|| format!("reading {list_path} body"))?;
    Ok(extract_first(&body, detail_pattern))
}

fn extract_first(html: &str, pattern: &str) -> Option<String> {
    let (prefix, suffix) = split_pattern(pattern)?;
    let doc = Html::parse_document(html);
    let selector = Selector::parse(&format!("a[href^=\"{prefix}\"]")).ok()?;
    for el in doc.select(&selector) {
        let Some(href) = el.value().attr("href") else {
            continue;
        };
        if let Some(id) = extract_id(href, prefix, suffix)
            && !id.is_empty()
            && !id.contains('/')
        {
            return Some(pattern.replace("{id}", &id));
        }
    }
    None
}

fn split_pattern(pattern: &str) -> Option<(&str, &str)> {
    let idx = pattern.find("{id}")?;
    let prefix = pattern.get(..idx)?;
    let suffix = pattern.get(idx + "{id}".len()..)?;
    Some((prefix, suffix))
}

fn extract_id<'a>(href: &'a str, prefix: &str, suffix: &str) -> Option<&'a str> {
    let path = href.split('?').next()?;
    let rest = path.strip_prefix(prefix)?;
    if suffix.is_empty() {
        if rest.contains('/') { None } else { Some(rest) }
    } else {
        rest.strip_suffix(suffix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_facture_items_pattern() {
        let html = r#"
          <a href="/factures/new">New</a>
          <a href="/factures/abc-1/items">Row</a>
          <a href="/factures/xyz-2/items">Another</a>
        "#;
        assert_eq!(
            extract_first(html, "/factures/{id}/items").as_deref(),
            Some("/factures/abc-1/items")
        );
    }

    #[test]
    fn matches_bare_detail_pattern() {
        let html = r#"
          <a href="/clients">Index</a>
          <a href="/clients/new">New</a>
          <a href="/clients/rec123">Row</a>
          <a href="/clients/rec456">Row 2</a>
        "#;
        assert_eq!(
            extract_first(html, "/clients/{id}").as_deref(),
            Some("/clients/rec123")
        );
    }

    #[test]
    fn empty_listing_returns_none() {
        assert_eq!(extract_first("<html></html>", "/clients/{id}"), None);
    }
}
