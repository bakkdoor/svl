use crate::db::{DBConnection, DBParams, DBResult, ToDataValue};

// get the top words by count across all texts that start with the given prefix
pub fn top_words_starting_with(db: &DBConnection, prefix: &str, limit: usize) -> DBResult {
    let query = r#"
    ?[word, sum(count), count(text_id)] := *Word{word,count,text_id},
      starts_with(word, $prefix),
      :sort -count(text_id), word :limit $limit
    "#;

    db.run_immutable(
        query,
        DBParams::from_iter(vec![
            ("prefix".into(), prefix.to_data_value()),
            ("limit".into(), limit.to_data_value()),
        ]),
    )
}

// get all texts that have a word starting with the given prefix
pub fn texts_with_word_starting_with(db: &DBConnection, prefix: &str) -> DBResult {
    let query = r#"
    ?[text_id, url, text] := *Text{text_id,url,text},
      ?[word, count] := *Word{word,count,text_id},
      starts_with(word, $prefix)
    "#;

    db.run_immutable(
        query,
        DBParams::from_iter(vec![("prefix".into(), prefix.to_data_value())]),
    )
}

// get all words that end with the given suffix
pub fn words_ending_with(db: &DBConnection, suffix: &str) -> DBResult {
    let query = r#"
    ?[word, sum(count), count(text_id)] := *Word{word,count,text_id},
      ends_with(word, $suffix),
      :sort -count(text_id), word
    "#;

    db.run_immutable(
        query,
        DBParams::from_iter(vec![("suffix".into(), suffix.to_data_value())]),
    )
}

// get all texts that have a word ending with the given suffix
pub fn texts_with_word_ending_with(db: &DBConnection, suffix: &str) -> DBResult {
    let query = r#"
    ?[text_id, url, text] := *Text{text_id,url,text},
      ?[word, count] := *Word{word,count,text_id},
      ends_with(word, $suffix)
    "#;

    db.run_immutable(
        query,
        DBParams::from_iter(vec![("suffix".into(), suffix.to_data_value())]),
    )
}

// get all words that contain the given substring
pub fn words_containing(db: &DBConnection, substring: &str) -> DBResult {
    let query = r#"
    ?[word, sum(count), count(text_id)] := *Word{word,count,text_id},
      str_includes(word, $substring),
      :sort -count(text_id), word
    "#;

    db.run_immutable(
        query,
        DBParams::from_iter(vec![("substring".into(), substring.to_data_value())]),
    )
}
