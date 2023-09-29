use std::{
    collections::{HashMap, HashSet},
    fmt::{Display, Formatter},
};

use serde_derive::{Deserialize, Serialize};

use crate::{
    db::{val, DBConnection, DBParams},
    text::{Text, TextId, Word},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stats {
    texts: Vec<Text>,
    word_count: usize,
    words: HashMap<Word, WordStats>,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            texts: Vec::new(),
            word_count: 0,
            words: HashMap::new(),
        }
    }

    pub fn unique_word_count(&self) -> usize {
        self.words.len()
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
        word_stats.count_text(text_id);
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
                ?[text_id, url, author_id, text] <- [$props];
                :put Text { text_id, author_id => url, text }
                ",
                DBParams::from_iter(vec![(
                    "props".into(),
                    val(vec![
                        val(text_id),
                        val(text_url),
                        val(author_id),
                        val(text.text.clone()),
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
                        val(vec![
                            val(word),
                            val(word_stats.count(text_id)),
                            val(text_id),
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
        writeln!(f, "Unique words: {}", self.unique_word_count())?;
        writeln!(f, "Texts: {}", self.texts.len())?;
        if std::env::var("SHOW_WORDS").is_ok() {
            writeln!(f, "Words:")?;
            for (word, stats) in &self.words {
                writeln!(f, "\t{} : {}", word, stats.global_count())?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordStats {
    text_ids: HashSet<TextId>,
    word: Word,
    count: HashMap<TextId, usize>,
}

impl WordStats {
    pub fn new(text_id: TextId, word: Word) -> Self {
        Self {
            text_ids: HashSet::from_iter(vec![text_id]),
            word,
            count: HashMap::new(),
        }
    }

    pub fn count_text(&mut self, text_id: TextId) {
        self.text_ids.insert(text_id);
        self.incr_count(text_id);
    }

    pub fn count(&self, text_id: &TextId) -> usize {
        self.count.get(text_id).copied().unwrap_or(0)
    }

    pub fn global_count(&self) -> usize {
        self.count.values().sum()
    }

    pub fn incr_count(&mut self, text_id: TextId) {
        let count = self.count.entry(text_id).or_insert(0);
        *count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_text() {
        let mut stats = Stats::new();
        let text = Text::new(
            "URL".into(),
            "Salvē amīcē, quōmodo tē hodiē habēs? Tē nunc vidēre possum.".into(),
        );
        stats.add_text(text);

        assert_eq!(stats.texts.len(), 1);
        assert_eq!(stats.word_count, 10);
        assert_eq!(stats.unique_word_count(), 9);
        assert_eq!(stats.words.get(&"salvē".into()).unwrap().global_count(), 1);
        assert_eq!(stats.words.get(&"amīcē".into()).unwrap().global_count(), 1);
        assert_eq!(
            stats.words.get(&"quōmodo".into()).unwrap().global_count(),
            1
        );
        assert_eq!(stats.words.get(&"tē".into()).unwrap().global_count(), 2);
        assert_eq!(stats.words.get(&"hodiē".into()).unwrap().global_count(), 1);
        assert_eq!(stats.words.get(&"habēs".into()).unwrap().global_count(), 1);
        assert_eq!(stats.words.get(&"nunc".into()).unwrap().global_count(), 1);
        assert_eq!(stats.words.get(&"vidēre".into()).unwrap().global_count(), 1);
        assert_eq!(stats.words.get(&"possum".into()).unwrap().global_count(), 1);

        let text = Text::new(
            "URL".into(),
            "Quid nunc? Tibi iam respondēre possum!".into(),
        );
        stats.add_text(text);

        assert_eq!(stats.texts.len(), 2);
        assert_eq!(stats.word_count, 16);
        assert_eq!(stats.unique_word_count(), 13);
        assert_eq!(stats.words.get(&"quid".into()).unwrap().global_count(), 1);
        assert_eq!(stats.words.get(&"possum".into()).unwrap().global_count(), 2);
    }
}
