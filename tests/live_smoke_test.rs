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
use live::discover::first_detail_path;
use live::session::{AuthedSession, login};

/// Load a listing page then a representative detail page, asserting both
/// return 2xx and load cleanly in a browser (no JS errors).
///
/// The detail page is discovered by scraping the listing — tests don't depend
/// on specific ids, keeping them resilient to evolving data.
async fn assert_listing_and_detail_load(list_path: &str, detail_pattern: &str) -> Result<()> {
    let base = live::base_url();
    let (client, session) = login(&base).await?;
    let browser = TestBrowser::launch().await?;

    let list_url = base.join(list_path)?;
    check_http_ok(&client, &session, &list_url).await?;
    assert_page_loads_cleanly(&browser, &base, &session, &list_url).await?;

    let Some(detail_path) =
        first_detail_path(&client, &session, &base, list_path, detail_pattern).await?
    else {
        eprintln!("skipping detail check: {list_path} listing is empty");
        return Ok(());
    };
    let detail_url = base.join(&detail_path)?;
    check_http_ok(&client, &session, &detail_url).await?;
    assert_page_loads_cleanly(&browser, &base, &session, &detail_url).await?;

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
    assert_listing_and_detail_load("/factures", "/factures/{id}/items").await
}

#[tokio::test]
#[ignore = "requires live Scala app at LMAH_BASE_URL (default http://localhost:8080)"]
async fn clients_pages_load() -> Result<()> {
    assert_listing_and_detail_load("/clients", "/clients/{id}").await
}

#[tokio::test]
#[ignore = "requires live Scala app at LMAH_BASE_URL (default http://localhost:8080)"]
async fn events_pages_load() -> Result<()> {
    assert_listing_and_detail_load("/events", "/events/{id}").await
}
