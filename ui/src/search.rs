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
