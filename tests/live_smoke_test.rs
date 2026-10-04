//! Live smoke tests against a running instance of the original Scala app.
//!
//! Run with:
//!   cargo test --test live_smoke_test -- --ignored --nocapture
//! Point at a different host with:
//!   LMAH_BASE_URL=http://host:port cargo test --test live_smoke_test -- --ignored

mod live;

use anyhow::Result;
use url::Url;

use live::browser::{TestBrowser, assert_page_loads_cleanly};
use live::discover::first_id;
use live::session::{AuthedSession, login};

/// Load a listing page then one or more representative detail pages that
/// share a single discovered id, asserting each returns 2xx and loads cleanly
/// in a browser (no JS errors).
///
/// The id is scraped from the listing using the first pattern in
/// `detail_patterns`; the same id is substituted into every pattern. Tests
/// stay data-agnostic.
async fn assert_pages_load(list_path: &str, detail_patterns: &[&str]) -> Result<()> {
    let base = live::base_url();
    let (client, session) = login(&base).await?;
    let browser = TestBrowser::launch().await?;

    let list_url = base.join(list_path)?;
    check_http_ok(&client, &session, &list_url).await?;
    assert_page_loads_cleanly(&browser, &base, &session, &list_url).await?;

    let Some(discovery_pattern) = detail_patterns.first() else {
        return Ok(());
    };
    let Some(id) = first_id(&client, &session, &base, list_path, discovery_pattern).await? else {
        eprintln!("skipping detail checks: {list_path} listing is empty");
        return Ok(());
    };

    for pattern in detail_patterns {
        let path = pattern.replace("{id}", &id);
        let url = base.join(&path)?;
        check_http_ok(&client, &session, &url).await?;
        assert_page_loads_cleanly(&browser, &base, &session, &url).await?;
    }

    Ok(())
}

async fn check_http_ok(client: &reqwest::Client, session: &AuthedSession, url: &Url) -> Result<()> {
    let resp = session.apply(client.get(url.clone())).send().await?;
    anyhow::ensure!(
        resp.status().is_success(),
        "GET {url} returned {}",
        resp.status()
    );
    Ok(())
}

/// Download a CSV report and sanity-check it: 2xx, text/csv content type, and
/// a body that parses as CSV with at least a header row.
async fn check_csv_download(
    client: &reqwest::Client,
    session: &AuthedSession,
    url: &Url,
) -> Result<()> {
    let resp = session.apply(client.get(url.clone())).send().await?;
    anyhow::ensure!(
        resp.status().is_success(),
        "GET {url} returned {}",
        resp.status()
    );
    let ct = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    anyhow::ensure!(
        ct.starts_with("text/csv"),
        "GET {url} returned non-CSV content-type: {ct:?}"
    );
    let body = resp.bytes().await?;
    anyhow::ensure!(!body.is_empty(), "GET {url} returned empty body");
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(body.as_ref());
    let header = rdr
        .records()
        .next()
        .ok_or_else(|| anyhow::anyhow!("GET {url}: CSV had no rows"))??;
    anyhow::ensure!(!header.is_empty(), "GET {url}: CSV header row was empty");
    Ok(())
}

#[tokio::test]
#[ignore = "requires live Scala app at LMAH_BASE_URL (default http://localhost:8080)"]
async fn factures_pages_load() -> Result<()> {
    assert_pages_load(
        "/factures",
        &["/factures/{id}/items", "/factures/{id}/transactions"],
    )
    .await
}

#[tokio::test]
#[ignore = "requires live Scala app at LMAH_BASE_URL (default http://localhost:8080)"]
async fn clients_pages_load() -> Result<()> {
    assert_pages_load("/clients", &["/clients/{id}"]).await
}

#[tokio::test]
#[ignore = "requires live Scala app at LMAH_BASE_URL (default http://localhost:8080)"]
async fn events_pages_load() -> Result<()> {
    assert_pages_load("/events", &["/events/{id}"]).await
}

#[tokio::test]
#[ignore = "requires live Scala app at LMAH_BASE_URL (default http://localhost:8080)"]
async fn admin_pages_load() -> Result<()> {
    let base = live::base_url();
    let (client, session) = login(&base).await?;
    let browser = TestBrowser::launch().await?;

    let admin_url = base.join("/admin")?;
    check_http_ok(&client, &session, &admin_url).await?;
    assert_page_loads_cleanly(&browser, &base, &session, &admin_url).await?;

    for report in ["/admin/paiements-report", "/admin/factures-report"] {
        let url = base.join(report)?;
        check_csv_download(&client, &session, &url).await?;
    }

    Ok(())
}
