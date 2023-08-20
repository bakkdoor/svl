use std::fmt::Display;

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TextId(usize);

impl From<usize> for TextId {
    fn from(idx: usize) -> Self {
        Self(idx)
    }
}

impl Display for TextId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
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
            .filter_map(|s| Self::trim_latin_word(s))
    }

    pub fn trim_latin_word(word: &str) -> Option<Word> {
        let trimmed = word.clone().trim();
        let trimmed = trimmed.replace("<p>", "").replace("</p>", "");
        let trimmed = trimmed.replace(
            &[
                '(', ')', '{', '}', '<', '>', '/', '+', '-', '*', ',', '·', '\'', '\"', '.', ';',
                ':', '\'',
            ][..],
            "",
        );

        let char = trimmed.chars().nth(0);
        if trimmed.is_empty() || char.is_none() || char.is_some() && !char.unwrap().is_alphabetic()
        {
            return None;
        }

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
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl From<&str> for Word {
    fn from(s: &str) -> Self {
        Self(s.into())
    }
}
