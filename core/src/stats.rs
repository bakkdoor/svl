use std::collections::HashMap;

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
// - the (number of) words found in each text
// - the (number of) unique words found in each text
// - the (number of) words found in each text that are not found in the other texts
pub struct Stats {
    pub word_count: usize,
    pub unique_word_count: usize,
    pub unique_words_not_in_other_texts_count: usize,

    pub words: HashMap<Word, WordStats>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordStats {
    pub word: Word,
    pub count: usize,
}
