use std::sync::Arc;

use serde_derive::{Deserialize, Serialize};

use crate::text::Text;

#[derive(Debug)]
pub struct HttpStatsClient {
    client: reqwest::Client,
    semaphore: Arc<tokio::sync::Semaphore>,
}

impl HttpStatsClient {
    const BASE_URL: &'static str = "https://thelatinlibrary.com/";
    const MAX_CONCURRENT_REQUESTS: usize = 25;

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

    pub async fn get_authors(&self) -> crate::Result<Vec<AuthorInfo>> {
        let _permit = self.semaphore.acquire().await?;
        let html_text = self.client.get(Self::BASE_URL).send().await?.text().await?;

        let html = scraper::Html::parse_document(&html_text);
        let mut authors = Vec::new();

        let selector =
            scraper::Selector::parse("form[name=myform] select[name=dest] option").unwrap();

        for author in html.select(&selector) {
            // <option value="$URL">$NAME</option>
            let author_info = AuthorInfo {
                name: author.inner_html().trim().into(),
                url: Self::path_to_url(author.value().attr("value").unwrap()),
                texts: Vec::new(),
            };
            authors.push(author_info);
        }

        Ok(authors)
    }

    pub fn path_to_url(path: &str) -> String {
        format!("{}{}", Self::BASE_URL, path.trim())
    }

    pub async fn get_texts(&self, author_info: &AuthorInfo) -> crate::Result<Vec<TextInfo>> {
        let permit = self.semaphore.acquire().await?;
        let html_text = self
            .client
            .get(author_info.url.clone())
            .send()
            .await?
            .text()
            .await?;
        drop(permit);

        let html = scraper::Html::parse_document(&html_text);
        let mut text_infos = Vec::new();

        let selector = scraper::Selector::parse("div.work table tr td a").unwrap();

        for txt in html.select(&selector) {
            let text_info = TextInfo {
                name: txt.inner_html().trim().into(),
                url: Self::path_to_url(txt.value().attr("href").unwrap().trim()),
            };
            text_infos.push(text_info);
        }

        Ok(text_infos)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorInfo {
    pub name: String,
    pub url: String,
    pub texts: Vec<TextInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextInfo {
    pub name: String,
    pub url: String,
}
