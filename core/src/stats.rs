use std::{collections::HashMap, sync::Arc};

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Word(String);

impl<S: ToString> From<S> for Word {
    fn from(s: S) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// Stats about Latin words found in various texts
// keeps track of:
// - the (number of) words found in all texts
// - the (number of) unique words found in all texts
pub struct Stats {
    pub word_count: usize,
    pub unique_word_count: usize,
    pub words: HashMap<Word, WordStats>,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            word_count: 0,
            unique_word_count: 0,
            words: HashMap::new(),
        }
    }

    pub fn add_word(&mut self, word: Word) {
        self.word_count += 1;
        let word_stats = self
            .words
            .entry(word.clone())
            .or_insert_with(|| WordStats { word, count: 0 });
        word_stats.count += 1;
        if word_stats.count == 1 {
            self.unique_word_count += 1;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordStats {
    pub word: Word,
    pub count: usize,
}

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

    pub async fn fetch_text(&self, text_url: &str) -> crate::Result<String> {
        let permit = self.semaphore.acquire().await?;
        let text = self.client.get(text_url).send().await?.text().await?;
        drop(permit);
        Ok(text)
    }
}
