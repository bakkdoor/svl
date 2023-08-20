use std::sync::Arc;

use crate::text::Text;

#[derive(Debug)]
pub struct HttpStatsClient {
    client: reqwest::Client,
    semaphore: Arc<tokio::sync::Semaphore>,
}

impl HttpStatsClient {
    const MAX_CONCURRENT_REQUESTS: usize = 10;

    pub fn new() -> crate::Result<Self> {
        let client = reqwest::Client::builder().https_only(true).build()?;
        // allow max of MAX_CONCURRENT_REQUESTS concurrent requests using this http client pool
        let semaphore = Arc::new(tokio::sync::Semaphore::new(Self::MAX_CONCURRENT_REQUESTS));

        Ok(Self { client, semaphore })
    }

    pub async fn fetch_text(&self, text_url: &str) -> crate::Result<Text> {
        let _permit = self.semaphore.acquire().await?;
        let text = self.client.get(text_url).send().await?.text().await?;
        let text = Text::new(text_url.into(), text);
        Ok(text)
    }
}
