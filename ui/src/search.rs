use svl_core::db::{DBError, DBParams, NamedRows};

use crate::errors::SearchError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchKind {
    Author,
    Text,
    #[default]
    Word,
}

impl SearchKind {
    pub fn all_kinds() -> Vec<SearchKind> {
        vec![SearchKind::Word, SearchKind::Author, SearchKind::Text]
    }
}

impl std::fmt::Display for SearchKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchKind::Author => write!(f, "Author"),
            SearchKind::Text => write!(f, "Text"),
            SearchKind::Word => write!(f, "Word"),
        }
    }
}

pub type SearchModeQuery = (String, DBParams);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchMode {
    #[default]
    StartsWith,
    EndsWith,
    Contains,
    IsEqual,
    IsNotEqual,
}

impl SearchMode {
    pub fn all_modes() -> Vec<SearchMode> {
        vec![
            SearchMode::StartsWith,
            SearchMode::EndsWith,
            SearchMode::Contains,
            SearchMode::IsEqual,
            SearchMode::IsNotEqual,
        ]
    }
}

impl SearchMode {
    pub fn query(&self, var: &str, term: String) -> SearchModeQuery {
        let func_name = match self {
            SearchMode::StartsWith => "starts_with",
            SearchMode::EndsWith => "ends_with",
            SearchMode::Contains => "str_includes",
            SearchMode::IsEqual => "eq",
            SearchMode::IsNotEqual => "neq",
        };
        let code = format!("{}({}, $term)", func_name, var);
        let params = DBParams::from_iter(vec![("term".into(), term.into())]);
        (code, params)
    }
}

impl std::fmt::Display for SearchMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchMode::StartsWith => write!(f, "starts with"),
            SearchMode::EndsWith => write!(f, "ends with"),
            SearchMode::Contains => write!(f, "contains"),
            SearchMode::IsEqual => write!(f, "is equal to"),
            SearchMode::IsNotEqual => write!(f, "is not equal to"),
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
    kind: SearchKind,
    rows: NamedRows,
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

    pub fn position(&self, column: &str) -> Result<usize, SearchError> {
        self.rows
            .headers
            .iter()
            .position(|s| s == column)
            .ok_or(SearchError::missing_column(column))
    }
}

impl TryFrom<SearchRows> for Vec<svl_core::text::Author> {
    type Error = SearchError;

    fn try_from(sr: SearchRows) -> Result<Self, Self::Error> {
        let name = sr.position("name")?;
        let url = sr.position("url")?;
        let rows = sr.rows;

        let authors = rows
            .rows
            .into_iter()
            .enumerate()
            .map(|(author_id, row)| {
                let name = row.get(name).unwrap().get_str().unwrap().to_string();
                let url = row.get(url).unwrap().get_str().unwrap().to_string();

                svl_core::text::Author {
                    author_id,
                    name,
                    url,
                }
            })
            .collect();

        Ok(authors)
    }
}

impl TryFrom<SearchRows> for Vec<svl_core::text::Text> {
    type Error = SearchError;

    fn try_from(sr: SearchRows) -> Result<Self, Self::Error> {
        let author_id = sr.position("author_id")?;
        let text = sr.position("text")?;
        let text_id = sr.position("text_id")?;
        let url = sr.position("url")?;
        let rows = sr.rows;

        let texts = rows
            .rows
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
            .collect();

        Ok(texts)
    }
}

impl TryFrom<SearchRows> for Vec<svl_core::text::Word> {
    type Error = SearchError;

    fn try_from(sr: SearchRows) -> Result<Self, Self::Error> {
        let word = sr.position("word")?;
        let rows = sr.rows;

        let words = rows
            .rows
            .into_iter()
            .map(|row| row.get(word).unwrap().get_str().unwrap().to_string().into())
            .collect();

        Ok(words)
    }
}
