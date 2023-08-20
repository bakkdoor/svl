use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TextId(usize);

impl From<usize> for TextId {
    fn from(idx: usize) -> Self {
        Self(idx)
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

    pub fn words(&self) -> impl Iterator<Item = Word> + '_ {
        self.text.split_whitespace().map(|s| s.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Word(String);

impl<S: ToString> From<S> for Word {
    fn from(s: S) -> Self {
        Self(s.to_string())
    }
}
