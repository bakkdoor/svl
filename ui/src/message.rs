use crate::search::{SearchKind, SearchResult};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Message {
    Closed,
    InputChanged(String),
    Search,
    SearchKindChanged(SearchKind),
    SearchCompleted(SearchResult),
}
