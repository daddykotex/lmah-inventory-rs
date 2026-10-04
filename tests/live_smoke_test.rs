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
