use serde_derive::{Deserialize, Serialize};
use std::fmt::Display;

use crate::db::{DataValue, Num};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TextId(usize);

impl From<&TextId> for DataValue {
    fn from(id: &TextId) -> Self {
        DataValue::Num(Num::Int(id.0 as i64))
    }
}

impl From<usize> for TextId {
    fn from(idx: usize) -> Self {
        Self(idx)
    }
}

impl Display for TextId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Text {
    pub id: Option<TextId>,
    pub url: String,
    pub text: String,
}

impl Text {
    pub fn new(url: String, text: String) -> Self {
        Self {
            id: None,
            url,
            text,
        }
    }

    pub fn set_id(&mut self, id: TextId) {
        self.id = Some(id);
    }

    pub fn words(&self) -> impl Iterator<Item = Word> + '_ {
        self.text
            .split_whitespace()
            .filter_map(Self::trim_latin_word)
    }

    pub fn trim_latin_word(word: &str) -> Option<Word> {
        let trimmed = word.trim().replace("&nbsp;", " ");
        if trimmed.is_empty() {
            return None;
        }

        // remove all non-alphabetic characters
        let trimmed = trimmed
            .chars()
            .filter(|c| c.is_alphabetic())
            .collect::<String>();

        let trimmed = scraper::Html::parse_fragment(&trimmed)
            .root_element()
            .text()
            .collect::<String>();

        Some(Word(trimmed))
    }

    pub fn is_latin_word(word: &Word) -> bool {
        word.0.chars().all(|c| c.is_alphabetic())
    }
}

impl<Url: ToString, Txt: ToString> From<(Url, Txt)> for Text {
    fn from((url, text): (Url, Txt)) -> Self {
        Self::new(url.to_string(), text.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Word(String);

impl Word {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl From<&str> for Word {
    fn from(s: &str) -> Self {
        Self(s.into())
    }
}

impl From<Word> for DataValue {
    fn from(w: Word) -> Self {
        DataValue::Str(w.0.into())
    }
}

impl From<&Word> for DataValue {
    fn from(w: &Word) -> Self {
        DataValue::Str(w.0.clone().into())
    }
}
