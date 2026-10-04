use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use chromiumoxide::cdp::browser_protocol::log::{EventEntryAdded, LogEntryLevel};
use chromiumoxide::cdp::browser_protocol::network::{
    CookieParam, EventLoadingFailed, SetCookiesParams,
};
use chromiumoxide::cdp::js_protocol::runtime::{
    ConsoleApiCalledType, EventConsoleApiCalled, EventExceptionThrown,
};
use chromiumoxide::{Browser, BrowserConfig};
use futures_util::StreamExt;
use url::Url;

use super::session::AuthedSession;

pub struct TestBrowser {
    browser: Browser,
    _handler: tokio::task::JoinHandle<()>,
}

impl TestBrowser {
    pub async fn launch() -> Result<Self> {
        // Unique profile dir per launch so parallel test runs don't collide on
        // Chromium's SingletonLock.
        let data_dir = std::env::temp_dir().join(format!(
            "lmah-live-smoke-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&data_dir).context("creating browser data dir")?;
        let config = BrowserConfig::builder()
            .user_data_dir(&data_dir)
            .build()
            .map_err(|e| anyhow::anyhow!("browser config: {e}"))?;
        let (browser, mut handler) = Browser::launch(config)
            .await
            .context("launching headless chromium — is chrome installed?")?;
        let handle = tokio::spawn(async move { while let Some(_evt) = handler.next().await {} });
        Ok(Self {
            browser,
            _handler: handle,
        })
    }
}

#[derive(Default, Debug)]
struct PageErrors {
    exceptions: Vec<String>,
    console_errors: Vec<String>,
    log_errors: Vec<String>,
    network_failures: Vec<String>,
}

impl PageErrors {
    fn is_empty(&self) -> bool {
        self.exceptions.is_empty()
            && self.console_errors.is_empty()
            && self.log_errors.is_empty()
            && self.network_failures.is_empty()
    }

    fn format(&self) -> String {
        let mut lines = Vec::new();
        for e in &self.exceptions {
            lines.push(format!("  [exception] {e}"));
        }
        for e in &self.console_errors {
            lines.push(format!("  [console.error] {e}"));
        }
        for e in &self.log_errors {
            lines.push(format!("  [log] {e}"));
        }
        for e in &self.network_failures {
            lines.push(format!("  [network] {e}"));
        }
        lines.join("\n")
    }
}

/// Load `target_url` in a fresh page using the authenticated session, and
/// assert there were no JS errors.
pub async fn assert_page_loads_cleanly(
    tb: &TestBrowser,
    base: &Url,
    session: &AuthedSession,
    target_url: &Url,
) -> Result<()> {
    let page = tb
        .browser
        .new_page("about:blank")
        .await
        .context("opening blank page")?;

    let cookie_params: Vec<CookieParam> = session
        .cookies
        .iter()
        .map(|(name, value)| {
            let mut c = CookieParam::new(name.clone(), value.clone());
            c.url = Some(base.as_str().to_string());
            c
        })
        .collect();
    if !cookie_params.is_empty() {
        page.execute(SetCookiesParams {
            cookies: cookie_params,
        })
        .await
        .context("setting cookies on browser")?;
    }

    let errors = Arc::new(Mutex::new(PageErrors::default()));

    let mut exc_stream = page
        .event_listener::<EventExceptionThrown>()
        .await
        .context("subscribing to exceptionThrown")?;
    let mut console_stream = page
        .event_listener::<EventConsoleApiCalled>()
        .await
        .context("subscribing to consoleAPICalled")?;
    let mut log_stream = page
        .event_listener::<EventEntryAdded>()
        .await
        .context("subscribing to log.entryAdded")?;
    let mut net_stream = page
        .event_listener::<EventLoadingFailed>()
        .await
        .context("subscribing to loadingFailed")?;

    let errors_exc = errors.clone();
    let t_exc = tokio::spawn(async move {
        while let Some(ev) = exc_stream.next().await {
            let msg = ev.exception_details.text.clone();
            let desc = ev
                .exception_details
                .exception
                .as_ref()
                .and_then(|e| e.description.clone())
                .unwrap_or_default();
            let full = if desc.is_empty() {
                msg
            } else {
                format!("{msg}: {desc}")
            };
            if let Ok(mut g) = errors_exc.lock() {
                g.exceptions.push(full);
            }
        }
    });

    let errors_con = errors.clone();
    let t_con = tokio::spawn(async move {
        while let Some(ev) = console_stream.next().await {
            if !matches!(ev.r#type, ConsoleApiCalledType::Error) {
                continue;
            }
            let text = ev
                .args
                .iter()
                .filter_map(|a| a.value.as_ref().map(|v| v.to_string()))
                .collect::<Vec<_>>()
                .join(" ");
            if let Ok(mut g) = errors_con.lock() {
                g.console_errors.push(text);
            }
        }
    });

    let errors_log = errors.clone();
    let t_log = tokio::spawn(async move {
        while let Some(ev) = log_stream.next().await {
            if !matches!(ev.entry.level, LogEntryLevel::Error) {
                continue;
            }
            if let Ok(mut g) = errors_log.lock() {
                g.log_errors.push(ev.entry.text.clone());
            }
        }
    });

    let errors_net = errors.clone();
    let t_net = tokio::spawn(async move {
        while let Some(ev) = net_stream.next().await {
            if ev.canceled.unwrap_or(false) {
                continue;
            }
            if let Ok(mut g) = errors_net.lock() {
                g.network_failures
                    .push(format!("{} ({:?})", ev.error_text, ev.r#type));
            }
        }
    });

    page.goto(target_url.as_str())
        .await
        .context("navigating to target url")?
        .wait_for_navigation()
        .await
        .context("waiting for navigation")?;

    // Give events a brief window to flush, then stop listeners.
    tokio::time::sleep(Duration::from_millis(500)).await;
    t_exc.abort();
    t_con.abort();
    t_log.abort();
    t_net.abort();

    let guard = errors
        .lock()
        .map_err(|_| anyhow::anyhow!("errors mutex poisoned"))?;
    if !guard.is_empty() {
        bail!("JS errors on {target_url}:\n{}", guard.format());
    }
    Ok(())
}
