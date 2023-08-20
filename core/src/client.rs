use std::sync::Arc;

use serde_derive::{Deserialize, Serialize};

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

    pub async fn get_authors(&self) -> crate::Result<Vec<AuthorInfo>> {
        let _permit = self.semaphore.acquire().await?;
        let html_text = self
            .client
            .get("https://thelatinlibrary.com/")
            .send()
            .await?
            .text()
            .await?;

        // // parse html and find tag
        // // <select name="dest" ..>

        let html = scraper::Html::parse_document(&html_text);
        let mut authors = Vec::new();

        // get all form select options for the form with name "myform" and select with name "dest"
        let selector =
            scraper::Selector::parse("form[name=myform] select[name=dest] option").unwrap();

        for author in html.select(&selector) {
            // option looks like this:
            // <option value="$URL">$NAME</option>
            let author_info = AuthorInfo {
                name: author.inner_html().trim().into(),
                url: author
                    .value()
                    .attr("value")
                    .unwrap()
                    .to_string()
                    .trim()
                    .into(),
                texts: Vec::new(),
            };
            authors.push(author_info);
        }

        Ok(authors)
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
