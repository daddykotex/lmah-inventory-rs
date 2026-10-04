use anyhow::{Context, Result, bail};
use reqwest::redirect::Policy;
use reqwest::{Client, StatusCode, header};
use url::Url;

#[derive(Clone, Debug)]
pub struct AuthedSession {
    pub cookies: Vec<(String, String)>,
}

impl AuthedSession {
    pub fn cookie_header(&self) -> String {
        self.cookies
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("; ")
    }

    pub fn apply(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if self.cookies.is_empty() {
            req
        } else {
            req.header(header::COOKIE, self.cookie_header())
        }
    }
}

fn build_client() -> Result<Client> {
    Client::builder()
        .redirect(Policy::none())
        .build()
        .context("failed to build reqwest client")
}

/// Perform the TEST_MODE dummy auth handshake, following redirects manually so
/// we can capture every Set-Cookie along the way (mirrors the Scala suite's
/// cookie-jar middleware).
pub async fn login(base: &Url) -> Result<(Client, AuthedSession)> {
    let client = build_client()?;
    let mut cookies: Vec<(String, String)> = Vec::new();
    let mut url = base
        .join("/signin/complete?state=dummy&code=dummy")
        .context("joining signin url")?;

    // Seed the state cookie the signin flow expects.
    cookies.push(("state".into(), "dummy".into()));

    for _ in 0..5 {
        let cookie_header = cookies
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("; ");
        let resp = client
            .get(url.clone())
            .header(header::COOKIE, cookie_header)
            .send()
            .await
            .context("signin request")?;

        for sc in resp.headers().get_all(header::SET_COOKIE).iter() {
            if let Ok(raw) = sc.to_str()
                && let Some((name, value)) = parse_set_cookie(raw)
            {
                cookies.retain(|(k, _)| k != &name);
                cookies.push((name, value));
            }
        }

        let status = resp.status();
        if status.is_redirection() {
            let Some(loc) = resp.headers().get(header::LOCATION) else {
                bail!("redirect without Location header");
            };
            let loc_str = loc.to_str().context("Location not utf-8")?;
            url = url.join(loc_str).context("joining redirect url")?;
            continue;
        }

        if status.is_success() {
            return Ok((client, AuthedSession { cookies }));
        }

        bail!(
            "signin flow returned {status} at {url} — is the original app running with TEST_MODE=true?"
        );
    }

    bail!("signin flow exceeded redirect budget");
}

fn parse_set_cookie(raw: &str) -> Option<(String, String)> {
    let first = raw.split(';').next()?.trim();
    let (name, value) = first.split_once('=')?;
    if name.is_empty() {
        return None;
    }
    Some((name.to_string(), value.to_string()))
}

#[expect(dead_code, reason = "kept for discoverability of status helper")]
pub fn is_ok(s: StatusCode) -> bool {
    s.is_success()
}
