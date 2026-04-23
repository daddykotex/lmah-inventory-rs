use anyhow::Result;
use axum::body::Bytes;
use futures_core::Stream;
use maud::Markup;
use reqwest::Client;
use reqwest::Result as RequestResult;
use std::collections::HashMap;

pub async fn print_to_pdf(
    client: &Client,
    api_key: &str,
    html: Markup,
) -> Result<impl Stream<Item = RequestResult<Bytes>> + use<>> {
    let html_rendered = html.into_string();
    let payload = HashMap::from([
        ("apiKey", api_key),
        ("MarginTop", "10"),
        ("value", &html_rendered),
    ]);

    Ok(client
        .post("https://api.html2pdfrocket.com/pdf")
        .form(&payload)
        .send()
        .await?
        .bytes_stream())
}
