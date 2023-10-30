use svl_core::db::{DBError, DBParams, NamedRows};

use crate::errors::{ExpectedType, SearchError};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Search {
    pub kind: SearchKind,
    pub term: String,
    pub mode: SearchMode,
    pub is_case_sensitive: bool,
}

impl Search {
    pub fn new(kind: SearchKind, term: String, mode: SearchMode, is_case_sensitive: bool) -> Self {
        Self {
            kind,
            term,
            mode,
            is_case_sensitive,
        }
    }

    pub fn query(&self, var: &str) -> SearchQuery {
        let (var, term) = self.var_and_term(var);
        let (code, params) = self.mode.query(var.as_str(), term);
        SearchQuery {
            kind: self.kind,
            code,
            params,
        }
    }

    fn var_and_term(&self, var: &str) -> (String, String) {
        if self.is_case_sensitive {
            (var.to_string(), self.term.clone())
        } else {
            (format!("lowercase({})", var), self.term.to_lowercase())
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub kind: SearchKind,
    pub code: String,
    pub params: DBParams,
}

pub type SearchModeQuery = (String, DBParams);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchMode {
    Contains,
    EndsWith,
    IsEqual,
    IsNotEqual,
    #[default]
    StartsWith,
}

impl SearchMode {
    pub fn all_modes() -> Vec<SearchMode> {
        vec![
            SearchMode::Contains,
            SearchMode::EndsWith,
            SearchMode::IsEqual,
            SearchMode::IsNotEqual,
            SearchMode::StartsWith,
        ]
    }
}

impl SearchMode {
    pub fn query(&self, var: &str, term: String) -> SearchModeQuery {
        let func_name = match self {
            SearchMode::Contains => "str_includes",
            SearchMode::EndsWith => "ends_with",
            SearchMode::IsEqual => "eq",
            SearchMode::IsNotEqual => "neq",
            SearchMode::StartsWith => "starts_with",
        };
        let code = format!("{}({}, $term)", func_name, var);
        let params = DBParams::from_iter(vec![("term".into(), term.into())]);
        (code, params)
    }
}

impl std::fmt::Display for SearchMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchMode::Contains => write!(f, "contains"),
            SearchMode::EndsWith => write!(f, "ends with"),
            SearchMode::IsEqual => write!(f, "is equal to"),
            SearchMode::IsNotEqual => write!(f, "is not equal to"),
            SearchMode::StartsWith => write!(f, "starts with"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchState<Result> {
    active_searches: Vec<Search>,
    is_case_sensitive: bool,
    search_term: String,
    search_results: Vec<Result>,
}

impl<Result> SearchState<Result> {
    pub fn search_term(&self) -> String {
        self.search_term.clone()
    }

    pub fn started_search(&mut self, search: Search) {
        self.active_searches.push(search);
    }

    pub fn ended_search(&mut self, search: &Search) {
        self.active_searches.retain(|s| s != search);
    }

    pub fn search_results_iter(&self) -> impl Iterator<Item = &Result> {
        self.search_results.iter()
    }

    pub fn search_results_count(&self) -> usize {
        self.search_results.len()
    }

    pub fn is_case_sensitive(&self) -> bool {
        self.is_case_sensitive
    }

    pub fn update_search(&mut self, term: &str) {
        self.search_term = term.to_string();
    }

    pub fn update_search_results(&mut self, rows: Vec<Result>) {
        self.search_results = rows;
    }

    pub fn update_case_sensitive(&mut self, is_case_sensitive: bool) {
        self.is_case_sensitive = is_case_sensitive;
    }

    pub fn is_searching(&self) -> bool {
        !self.active_searches.is_empty()
    }
}

impl<Result> Default for SearchState<Result> {
    fn default() -> Self {
        Self {
            active_searches: Vec::new(),
            is_case_sensitive: true,
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
    search: Search,
    rows: NamedRows,
}

impl SearchRows {
    pub fn new(search: Search, rows: NamedRows) -> Self {
        Self { search, rows }
    }

    pub fn search(&self) -> &Search {
        &self.search
    }

    pub fn rows(&self) -> &NamedRows {
        &self.rows
    }

    pub fn kind(&self) -> &SearchKind {
        &self.search.kind
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

        let mut authors = Vec::with_capacity(rows.rows.len());

        for (author_id, row) in rows.rows.into_iter().enumerate() {
            let name = row.get(name).ok_or(SearchError::missing_column("name"))?;
            let name = name
                .get_str()
                .map(|s| s.to_string())
                .ok_or(SearchError::invalid_type("name", ExpectedType::String))?;

            let url = row.get(url).ok_or(SearchError::missing_column("url"))?;
            let url = url
                .get_str()
                .map(|s| s.to_string())
                .ok_or(SearchError::invalid_type("url", ExpectedType::String))?;

            let author = svl_core::text::Author {
                author_id,
                name,
                url,
            };

            authors.push(author);
        }

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

        let mut texts = Vec::with_capacity(rows.rows.len());

        for row in rows.rows {
            let id = row.get(text_id).and_then(|x| x.get_int()).map(|i| i.into());

            let text = row.get(text).ok_or(SearchError::missing_column("text"))?;
            let text = text
                .get_str()
                .map(|x| x.to_string())
                .ok_or(SearchError::invalid_type("text", ExpectedType::String))?;

            let author_id = row
                .get(author_id)
                .ok_or(SearchError::missing_column("author_id"))?;
            let author_id = author_id
                .get_int()
                .map(|i| i as usize)
                .ok_or(SearchError::invalid_type("author_id", ExpectedType::Usize))?;

            let url = row.get(url).ok_or(SearchError::missing_column("url"))?;
            let url = url
                .get_str()
                .map(|x| x.to_string())
                .ok_or(SearchError::invalid_type("url", ExpectedType::String))?;

            let t = svl_core::text::Text {
                id,
                text,
                author_id: Some(author_id),
                url,
            };

            texts.push(t);
        }

        Ok(texts)
    }
}

impl TryFrom<SearchRows> for Vec<svl_core::text::Word> {
    type Error = SearchError;

    fn try_from(sr: SearchRows) -> Result<Self, Self::Error> {
        let word = sr.position("word")?;
        let rows = sr.rows;

        let mut words = Vec::with_capacity(rows.rows.len());

        for row in rows.rows {
            let word = row.get(word).ok_or(SearchError::missing_column("word"))?;
            let word = word
                .get_str()
                .map(|s| s.to_string())
                .ok_or(SearchError::invalid_type("word", ExpectedType::String))?;

            words.push(word.into());
        }

        Ok(words)
    }
}
