use crate::search::{SearchKind, SearchMode, SearchResult, SearchRows};
use svl_core::db::DBConnection;

#[allow(dead_code)]
pub async fn search_authors(
    db: DBConnection,
    term: String,
    search_mode: SearchMode,
) -> SearchResult {
    let (query, params) = search_mode.query("name", term);
    let script = format!(
        "?[word] :=
            *Author {{ name, url }},
            {}",
        query
    );
    let rows = db.run_immutable(&script, params).await?;
    Ok(SearchRows::new(SearchKind::Author, rows))
}

pub async fn search_words(db: DBConnection, term: String, search_mode: SearchMode) -> SearchResult {
    let (query, params) = search_mode.query("word", term);
    let script = format!(
        "?[word] :=
            *Word {{ word }},
            {}",
        query
    );
    let rows = db.run_immutable(&script, params).await?;
    Ok(SearchRows::new(SearchKind::Word, rows))
}

pub async fn search_texts(db: DBConnection, term: String, search_mode: SearchMode) -> SearchResult {
    let (query, params) = search_mode.query("word", term);
    let script = format!(
        "?[text_id, url, text, author_id] :=
            *Text {{ text_id, url, text, author_id }},
            *Word {{ word, text_id }},
            {}",
        query
    );
    let rows = db.run_immutable(&script, params).await?;
    Ok(SearchRows::new(SearchKind::Text, rows))
}
