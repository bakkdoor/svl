use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

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
    pub texts: Vec<Text>,
    pub word_count: usize,
    pub unique_word_count: usize,
    pub words: HashMap<Word, WordStats>,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            texts: Vec::new(),
            word_count: 0,
            unique_word_count: 0,
            words: HashMap::new(),
        }
    }

    pub fn add_text(&mut self, url: &str, text: &str) {
        let id = TextId::from(self.texts.len() + 1);
        self.texts.push(Text::new(id, url.into(), text.into()));
        for word in text.split_whitespace() {
            self.add_word(id, word.into());
        }
    }

    pub fn add_word(&mut self, text_id: TextId, word: Word) {
        self.word_count += 1;
        let word_stats = self
            .words
            .entry(word.clone())
            .or_insert_with(|| WordStats::new(text_id, word));
        word_stats.count += 1;
        if word_stats.count == 1 {
            self.unique_word_count += 1;
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TextId(usize);

impl From<usize> for TextId {
    fn from(idx: usize) -> Self {
        Self(idx)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Text {
    pub id: TextId,
    pub url: String,
    pub text: String,
}

impl Text {
    pub fn new(id: TextId, url: String, text: String) -> Self {
        Self { id, url, text }
    }

    pub fn words(&self) -> impl Iterator<Item = Word> + '_ {
        self.text.split_whitespace().map(|s| s.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordStats {
    pub text_ids: HashSet<TextId>,
    pub word: Word,
    pub count: usize,
}

impl WordStats {
    pub fn new(text_id: TextId, word: Word) -> Self {
        Self {
            text_ids: HashSet::from_iter(vec![text_id]),
            word,
            count: 0,
        }
    }
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
