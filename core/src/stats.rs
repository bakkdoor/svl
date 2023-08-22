use std::{
    collections::{HashMap, HashSet},
    fmt::{Display, Formatter},
};

use serde_derive::{Deserialize, Serialize};

use crate::text::{Text, TextId, Word};

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

    pub fn add_text(&mut self, text: &Text) {
        let id = TextId::from(self.texts.len() + 1);
        println!("Processing Text {}: {}", id, text.url);
        let mut text = text.clone();
        text.set_id(id);
        let words = text.words();
        self.texts.push(text);
        for word in words {
            self.add_word(id, word);
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

    pub fn merge(&mut self, other: &Self) {
        for text in &other.texts {
            self.add_text(&mut text.clone());
        }
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
                writeln!(f, "{}: {}", word, stats.count)?;
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
        let mut text = Text::new("URL".into(), "hello world test text".into());
        stats.add_text(&mut text);

        assert_eq!(stats.texts.len(), 1);
        assert_eq!(stats.word_count, 4);
        assert_eq!(stats.unique_word_count, 4);
        assert_eq!(stats.words.len(), 4);
        assert_eq!(stats.words.get(&"hello".into()).unwrap().count, 1);
        assert_eq!(stats.words.get(&"world".into()).unwrap().count, 1);
        assert_eq!(stats.words.get(&"test".into()).unwrap().count, 1);
        assert_eq!(stats.words.get(&"text".into()).unwrap().count, 1);

        let mut text = Text::new("URL".into(), "more text is here?!".into());
        stats.add_text(&mut text);

        assert_eq!(stats.texts.len(), 2);
        assert_eq!(stats.word_count, 8);
        assert_eq!(stats.unique_word_count, 7);
        assert_eq!(stats.words.len(), 7);
        assert_eq!(stats.words.get(&"hello".into()).unwrap().count, 1);
        assert_eq!(stats.words.get(&"text".into()).unwrap().count, 2);
    }
}
