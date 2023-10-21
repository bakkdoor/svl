use crate::search::{SearchKind, SearchResult, SearchRows};
use svl_core::db::{DBConnection, DBParams};

#[allow(dead_code)]
pub async fn search_authors(db: DBConnection, term: String) -> SearchResult {
    let script = r#"
    ?[name,url] :=
        *Author { name, url },
        str_includes(name, $name)
    "#;
    let params = DBParams::from_iter(vec![("name".into(), term.into())]);
    let rows = db.run_immutable(script, params).await?;
    Ok(SearchRows::new(SearchKind::Author, rows))
}

pub async fn search_words(db: DBConnection, term: String) -> SearchResult {
    let script = r#"
    ?[word] :=
        *Word { word },
        starts_with(word, $term)
    "#;
    let params = DBParams::from_iter(vec![("term".into(), term.into())]);
    let rows = db.run_immutable(script, params).await?;
    Ok(SearchRows::new(SearchKind::Word, rows))
}

pub async fn search_texts(db: DBConnection, term: String) -> SearchResult {
    let script = r#"
    ?[text_id, url, text, author_id] :=
        *Text { text_id, url, text, author_id },
        *Word { word, text_id },
        starts_with(word, $term)
    "#;
    let params = DBParams::from_iter(vec![("term".into(), term.into())]);
    let rows = db.run_immutable(script, params).await?;
    Ok(SearchRows::new(SearchKind::Text, rows))
}
