use crate::search::{SearchKind, SearchMode, SearchResult};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Message {
    Closed,
    InputChanged(String),
    Search,
    SearchKindChanged(SearchKind),
    SearchModeChanged(SearchMode),
    SearchCompleted(SearchResult),
}
