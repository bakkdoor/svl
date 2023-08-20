use std::collections::{HashMap, HashSet};

use serde_derive::{Deserialize, Serialize};

use crate::text::{Text, TextId, Word};

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

    pub fn add_text(&mut self, text: Text) {
        let id = TextId::from(self.texts.len() + 1);
        self.texts.push(text.clone());
        for word in text.words() {
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
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
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
