use anyhow::Result;
use axum::body::Bytes;
use futures_core::Stream;
use google_cloud_auth::signer::Signer;
use google_cloud_storage::builder::storage::SignedUrlBuilder;
use google_cloud_storage::client::Storage;
use google_cloud_storage::http;
use reqwest::Result as RequestResult;
use std::time::Duration;

use crate::server::utils::streaming::ReqwestStreamSource;

pub async fn bytes_to_storage(
    storage: &Storage,
    signer: &Signer,
    bucket_name: &str,
    file_name: &str,
    bytes: impl Stream<Item = RequestResult<Bytes>> + Send + Sync + 'static,
    content_type: Option<&str>,
) -> Result<String> {
    let bucket_name = format!("projects/_/buckets/{}", bucket_name);
    let source = ReqwestStreamSource::new(Box::pin(bytes));
    let default_content_type = "application/octet-stream";
    let req = storage
        .write_object(&bucket_name, file_name, source)
        .set_content_type(content_type.unwrap_or(default_content_type));
    let _ = req.send_buffered().await?;

    let url = SignedUrlBuilder::for_object(&bucket_name, file_name)
        .with_method(http::Method::GET)
        .with_expiration(Duration::from_secs(3600)) // 1 hour
        .sign_with(signer)
        .await?;

    Ok(url)
}
