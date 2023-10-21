use svl_core::db::{DBError, NamedRows};

use crate::errors::SearchError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchKind {
    Author,
    Text,
    #[default]
    Word,
}

impl std::fmt::Display for SearchKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchKind::Author => write!(f, "Authors"),
            SearchKind::Text => write!(f, "Texts"),
            SearchKind::Word => write!(f, "Words"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchState<Result> {
    pub search_term: String,
    pub search_results: Vec<Result>,
}

impl<Result> SearchState<Result> {
    pub fn update_search(&mut self, term: &str) {
        self.search_term = term.to_string();
        self.search_results = Vec::new();
    }

    pub fn search_term(&self) -> String {
        self.search_term.clone()
    }

    pub fn update_search_results(&mut self, rows: Vec<Result>) {
        self.search_results = rows;
    }
}

impl<Result> Default for SearchState<Result> {
    fn default() -> Self {
        Self {
            search_term: String::new(),
            search_results: Vec::new(),
        }
    }
}

pub type SearchResult = Result<SearchRows, SearchError>;

impl From<DBError> for SearchError {
    fn from(err: DBError) -> Self {
        SearchError::Db(err.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct SearchRows {
    pub kind: SearchKind,
    pub rows: NamedRows,
}

#[allow(dead_code)]
impl SearchRows {
    pub fn new(kind: SearchKind, rows: NamedRows) -> Self {
        Self { kind, rows }
    }

    pub fn rows(&self) -> &NamedRows {
        &self.rows
    }

    pub fn kind(&self) -> SearchKind {
        self.kind
    }
}

impl From<SearchRows> for Vec<svl_core::text::Author> {
    fn from(search_rows: SearchRows) -> Self {
        // use rows.headers to get index of author_id, name, url
        // use rows.rows to get the values based on index via rows.headers
        // use the values to create a vector of svl_core::text::Author

        let rows = search_rows.rows;

        let name = rows.headers.iter().position(|s| s == "name").unwrap();
        let url = rows.headers.iter().position(|s| s == "url").unwrap();

        rows.rows
            .into_iter()
            .enumerate()
            .map(|(author_id, row)| svl_core::text::Author {
                author_id,
                name: row.get(name).unwrap().get_str().unwrap().to_string(),
                url: row.get(url).unwrap().get_str().unwrap().to_string(),
            })
            .collect()
    }
}

impl From<SearchRows> for Vec<svl_core::text::Text> {
    fn from(search_rows: SearchRows) -> Self {
        let rows = search_rows.rows;

        let author_id = rows.headers.iter().position(|s| s == "author_id").unwrap();
        let text = rows.headers.iter().position(|s| s == "text").unwrap();
        let text_id = rows.headers.iter().position(|s| s == "text_id").unwrap();
        let url = rows.headers.iter().position(|s| s == "url").unwrap();

        rows.rows
            .into_iter()
            .map(|row| {
                let t = svl_core::text::Text {
                    id: row.get(text_id).unwrap().get_int().map(|i| i.into()),
                    text: row.get(text).unwrap().get_str().unwrap().to_string(),
                    author_id: row.get(author_id).unwrap().get_int().map(|i| i as usize),
                    url: row.get(url).unwrap().get_str().unwrap().to_string(),
                };

                t
            })
            .collect()
    }
}

impl From<SearchRows> for Vec<svl_core::text::Word> {
    fn from(search_rows: SearchRows) -> Self {
        let rows = search_rows.rows;

        let word = rows.headers.iter().position(|s| s == "word").unwrap();

        rows.rows
            .into_iter()
            .map(|row| row.get(word).unwrap().get_str().unwrap().to_string().into())
            .collect()
    }
}
