use crate::search::{Search, SearchResult, SearchRows};
use svl_core::db::DBConnection;

#[allow(dead_code)]
pub async fn search_authors(db: DBConnection, search: Search) -> SearchResult {
    let query = search.query("name");
    let script = format!(
        "?[name, url] :=
            *Author {{ name, url }},
            {}",
        query.code
    );
    let rows = db.run_immutable(&script, query.params).await?;
    Ok(SearchRows::new(search, rows))
}

pub async fn search_words(db: DBConnection, search: Search) -> SearchResult {
    let query = search.query("word");
    let script = format!(
        "?[word] :=
            *Word {{ word }},
            {}",
        query.code
    );
    let rows = db.run_immutable(&script, query.params).await?;
    Ok(SearchRows::new(search, rows))
}

pub async fn search_texts(db: DBConnection, search: Search) -> SearchResult {
    let query = search.query("word");
    let script = format!(
        "?[text_id, url, text, author_id] :=
            *Text {{ text_id, url, text, author_id }},
            *Word {{ word, text_id }},
            {}",
        query.code
    );
    let rows = db.run_immutable(&script, query.params).await?;
    Ok(SearchRows::new(search, rows))
}
