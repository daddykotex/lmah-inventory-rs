//! Live smoke tests against a running instance of the original Scala app.
//!
//! Run with:
//!   cargo test --test live_smoke_test -- --ignored --nocapture
//! Point at a different host with:
//!   LMAH_BASE_URL=http://host:port cargo test --test live_smoke_test -- --ignored

mod live;

use anyhow::Result;

use live::browser::{TestBrowser, assert_page_loads_cleanly};
use live::discover::first_facture_id;
use live::session::login;

#[tokio::test]
#[ignore = "requires live Scala app at LMAH_BASE_URL (default http://localhost:8080)"]
async fn factures_index_loads() -> Result<()> {
    let base = live::base_url();
    let (client, session) = login(&base).await?;
    let browser = TestBrowser::launch().await?;

    let target = base.join("/factures")?;
    let resp = session.apply(client.get(target.clone())).send().await?;
    assert!(
        resp.status().is_success(),
        "GET /factures returned {}",
        resp.status()
    );

    assert_page_loads_cleanly(&browser, &base, &session, &target).await
}

#[tokio::test]
#[ignore = "requires live Scala app at LMAH_BASE_URL (default http://localhost:8080)"]
async fn facture_items_page_loads() -> Result<()> {
    let base = live::base_url();
    let (client, session) = login(&base).await?;
    let browser = TestBrowser::launch().await?;

    let Some(id) = first_facture_id(&client, &session, &base).await? else {
        eprintln!("skipping: /factures listing is empty, cannot pick an id");
        return Ok(());
    };

    let target = base.join(&format!("/factures/{id}/items"))?;
    let resp = session.apply(client.get(target.clone())).send().await?;
    assert!(
        resp.status().is_success(),
        "GET /factures/{id}/items returned {}",
        resp.status()
    );

    assert_page_loads_cleanly(&browser, &base, &session, &target).await
}
