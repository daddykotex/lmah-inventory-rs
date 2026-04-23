use axum::body::Bytes;
use futures_core::Stream;
use futures_util::StreamExt;
use google_cloud_storage::streaming_source::StreamingSource;
use reqwest::Result as RequestResult;
use std::pin::Pin;
use std::result::Result as StdResult;

pub struct ReqwestStreamSource {
    inner: Pin<Box<dyn Stream<Item = RequestResult<Bytes>> + Send + Sync>>,
}

impl ReqwestStreamSource {
    pub fn new(inner: Pin<Box<dyn Stream<Item = RequestResult<Bytes>> + Send + Sync>>) -> Self {
        Self { inner }
    }
}


impl StreamingSource for ReqwestStreamSource {
    type Error = reqwest::Error;

    async fn next(&mut self) -> Option<StdResult<Bytes, Self::Error>> {
        self.inner.next().await
    }
}
