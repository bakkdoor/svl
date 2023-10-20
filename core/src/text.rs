use serde_derive::{Deserialize, Serialize};
use std::fmt::Display;

use crate::db::{DataValue, Num, ToDataValue};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Author {
    pub author_id: usize,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TextId(usize);

impl From<TextId> for DataValue {
    fn from(id: TextId) -> Self {
        DataValue::Num(Num::Int(id.0 as i64))
    }
}

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

impl From<i64> for TextId {
    fn from(idx: i64) -> Self {
        Self(idx as usize)
    }
}

impl Display for TextId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ToDataValue for TextId {
    fn to_data_value(&self) -> DataValue {
        DataValue::Num(Num::Int(self.0 as i64))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Text {
    pub id: Option<TextId>,
    pub url: String,
    pub text: String,
    pub author_id: Option<usize>,
}

impl Text {
    pub fn new(url: String, text: String) -> Self {
        Self {
            id: None,
            url,
            text,
            author_id: None,
        }
    }

    pub fn set_id(&mut self, id: TextId) {
        self.id = Some(id);
    }

    pub fn words(&self) -> impl Iterator<Item = Word> + '_ {
        self.text
            .split(Self::word_splitter)
            .filter_map(Self::trim_latin_word)
    }

    pub fn word_splitter(c: char) -> bool {
        c.is_whitespace() || c.is_ascii_punctuation() || !c.is_alphanumeric()
    }

    pub fn trim_latin_word(word: &str) -> Option<Word> {
        if word.starts_with('<') || word.starts_with('>') {
            return None;
        }

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
            .collect::<String>()
            .to_lowercase();

        Some(Word(trimmed))
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

    pub fn to_lowercase(&self) -> Self {
        Self(self.0.to_lowercase())
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

impl From<String> for Word {
    fn from(s: String) -> Self {
        Self(s)
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

impl ToDataValue for Word {
    fn to_data_value(&self) -> DataValue {
        DataValue::Str(self.0.clone().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(txt: &str) -> Text {
        Text::new("https://example.com".into(), txt.to_string())
    }

    #[test]
    fn test_words() {
        assert_eq!(
            text("Quī linguam Latīnam discere vult variīs modīs id facere potest.")
                .words()
                .collect::<Vec<_>>(),
            vec![
                Word::from("quī"),
                Word::from("linguam"),
                Word::from("latīnam"),
                Word::from("discere"),
                Word::from("vult"),
                Word::from("variīs"),
                Word::from("modīs"),
                Word::from("id"),
                Word::from("facere"),
                Word::from("potest")
            ]
        );

        assert_eq!(
            text(". Quōmodo est?").words().collect::<Vec<_>>(),
            vec![Word::from("quōmodo"), Word::from("est")]
        );

        assert_eq!(
            text(". Ita!?! Unde venis?").words().collect::<Vec<_>>(),
            vec![Word::from("ita"), Word::from("unde"), Word::from("venis")]
        );

        assert_eq!(
            text("Per variās terrās (et maria multa) iter faciēbant.")
                .words()
                .collect::<Vec<_>>(),
            vec![
                Word::from("per"),
                Word::from("variās"),
                Word::from("terrās"),
                Word::from("et"),
                Word::from("maria"),
                Word::from("multa"),
                Word::from("iter"),
                Word::from("faciēbant")
            ]
        );
    }

    #[test]
    fn test_trim_latin_word() {
        assert_eq!(Text::trim_latin_word(" a..."), Some(Word::from("a")));
        assert_eq!(Text::trim_latin_word(" AB "), Some(Word::from("ab")));
        assert_eq!(Text::trim_latin_word("  est!?."), Some(Word::from("est")));
        assert_eq!(Text::trim_latin_word(". Ita!"), Some(Word::from("ita")));
        assert_eq!(
            Text::trim_latin_word(" habemus "),
            Some(Word::from("habemus"))
        );
        assert_eq!(Text::trim_latin_word("<html>"), None);
        assert_eq!(Text::trim_latin_word("<body>"), None);
        assert_eq!(Text::trim_latin_word("<head>"), None);
        assert_eq!(Text::trim_latin_word("</html>"), None);
        assert_eq!(Text::trim_latin_word("</body>"), None);
        assert_eq!(Text::trim_latin_word("</head>"), None);
        assert_eq!(Text::trim_latin_word("<p>"), None);
        assert_eq!(Text::trim_latin_word("<br/>"), None);
    }
}
