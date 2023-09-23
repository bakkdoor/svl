use std::{
    collections::{HashMap, HashSet},
    fmt::{Display, Formatter},
};

use serde_derive::{Deserialize, Serialize};

use crate::{
    db::{int_val, list_val, DBConnection, DBParams},
    text::{Text, TextId, Word},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// Stats about Latin words found in various texts
// keeps track of:
// - the (number of) words found in all texts
// - the (number of) unique words found in all texts
pub struct Stats {
    texts: Vec<Text>,
    word_count: usize,
    unique_word_count: usize,
    words: HashMap<Word, WordStats>,
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

    pub fn add_text(&mut self, text: Text) {
        let id = TextId::from(self.texts.len() + 1);
        let words: Vec<Word> = text.words().collect();
        println!(
            "Processing Text {} ({} words): {}",
            id,
            words.len(),
            text.url
        );
        let mut text = text;
        text.set_id(id);
        self.texts.push(text.clone());
        for word in words {
            self.add_word(id, word);
        }
    }

    pub fn add_word(&mut self, text_id: TextId, word: Word) {
        if word.is_empty() {
            return;
        }
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

    pub fn merge(&mut self, other: &Self) {
        for text in &other.texts {
            self.add_text(text.clone());
        }
    }

    pub fn store_in_db(&self, db: &DBConnection) -> Result<(), Box<dyn std::error::Error>> {
        println!("Storing Stats in DB");
        let tx = db.multi_tx(true);

        for text in &self.texts {
            let text_id = text.id.expect("Text should have an id");
            let author_id = text.author_id.expect("Text should have an author id");
            let text_url = text.url.clone();

            tx.run_script(
                "
                ?[text_id, url, author_id] <- [$props];
                :put Text { text_id, author_id => url }
                ",
                DBParams::from_iter(vec![(
                    "props".into(),
                    list_val(vec![
                        text_id.into(),
                        text_url.clone().into(),
                        int_val(author_id as i64),
                    ]),
                )]),
            )?;
        }

        for (word, word_stats) in &self.words {
            for text_id in &word_stats.text_ids {
                tx.run_script(
                    "
                    ?[word, count, text_id] <- [$props];
                    :put Word { word, text_id => count }
                    ",
                    DBParams::from_iter(vec![(
                        "props".into(),
                        list_val(vec![
                            word.clone().into(),
                            int_val(word_stats.count as i64),
                            text_id.into(),
                        ]),
                    )]),
                )?;
            }
        }
        tx.commit()?;
        Ok(())
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Stats {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Total words: {}", self.word_count)?;
        writeln!(f, "Unique words: {}", self.unique_word_count)?;
        writeln!(f, "Texts: {}", self.texts.len())?;
        if std::env::var("SHOW_WORDS").is_ok() {
            writeln!(f, "Words:")?;
            for (word, stats) in &self.words {
                writeln!(f, "\t{} : {}", word, stats.count)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordStats {
    text_ids: HashSet<TextId>,
    word: Word,
    count: usize,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_text() {
        let mut stats = Stats::new();
        let text = Text::new("URL".into(), "hello world test text".into());
        stats.add_text(text);

        assert_eq!(stats.texts.len(), 1);
        assert_eq!(stats.word_count, 4);
        assert_eq!(stats.unique_word_count, 4);
        assert_eq!(stats.words.len(), 4);
        assert_eq!(stats.words.get(&"hello".into()).unwrap().count, 1);
        assert_eq!(stats.words.get(&"world".into()).unwrap().count, 1);
        assert_eq!(stats.words.get(&"test".into()).unwrap().count, 1);
        assert_eq!(stats.words.get(&"text".into()).unwrap().count, 1);

        let text = Text::new("URL".into(), "more text is here?!".into());
        stats.add_text(text);

        assert_eq!(stats.texts.len(), 2);
        assert_eq!(stats.word_count, 8);
        assert_eq!(stats.unique_word_count, 7);
        assert_eq!(stats.words.len(), 7);
        assert_eq!(stats.words.get(&"hello".into()).unwrap().count, 1);
        assert_eq!(stats.words.get(&"text".into()).unwrap().count, 2);
    }
}
