use std::str::FromStr;

use crate::{
    db::{DBConnection, DBError, DBParams, DataValue, NamedRows, ToDataValue},
    text::TextId,
};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum QueryError {
    #[error("DB Error: {0}")]
    DBError(crate::db::DBError),

    #[error("Unknown Query: {0}")]
    UnknownQuery(String),

    #[error("Missing args for {0}: {1} expected but only {2} provided")]
    MissingArgs(String, usize, usize),

    #[error("Empty query")]
    EmptyQuery,

    #[error("Missing query")]
    MissingCommand,

    #[error("Unmatched quotes")]
    UnmatchedQuotes,
}

pub type QueryResult = Result<NamedRows, QueryError>;

impl From<DBError> for QueryError {
    fn from(error: DBError) -> Self {
        Self::DBError(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    pub cmd: String,
    pub args: Args,
}

impl Query {
    pub fn new(cmd: String, args: Vec<String>) -> Self {
        Self {
            cmd,
            args: Args { args },
        }
    }

    pub fn parse(query: &str) -> Result<Self, QueryError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(QueryError::EmptyQuery);
        }

        let chars = query.chars().peekable();
        let mut cmd = String::new();
        let mut args = Args::new();
        let mut current_arg = String::new();
        let mut in_quotes = false;

        for c in chars {
            match c {
                ' ' | '\t' if !in_quotes => {
                    if !current_arg.is_empty() {
                        if cmd.is_empty() {
                            cmd = current_arg;
                        } else {
                            args.push(current_arg);
                        }
                        current_arg = String::new();
                    }
                }
                '"' => {
                    in_quotes = !in_quotes;
                    if !in_quotes && !current_arg.is_empty() {
                        if cmd.is_empty() {
                            return Err(QueryError::MissingCommand);
                        }
                        args.push(current_arg);
                        current_arg = String::new();
                    }
                }
                _ => current_arg.push(c),
            }
        }

        if in_quotes {
            return Err(QueryError::UnmatchedQuotes);
        }

        if !current_arg.is_empty() {
            if cmd.is_empty() {
                cmd = current_arg;
            } else {
                args.push(current_arg);
            }
        }

        if cmd.is_empty() {
            return Err(QueryError::MissingCommand);
        }

        Ok(Self { cmd, args })
    }

    pub async fn eval(&self, db: &DBConnection) -> QueryResult {
        let Query { cmd, args } = self;
        let cmd_str = cmd.as_str();
        let cmd = cmd.clone();

        match cmd_str {
            "help" => print_help(),
            "top" => {
                if args.is_empty() {
                    return Err(QueryError::MissingArgs(cmd, 1, args.len()));
                }
                let prefix = args.get(0).unwrap();
                let limit = args.optional_at(1);
                top_words_starting_with(db, prefix, limit).await
            }
            "top-ends" => {
                if args.is_empty() {
                    return Err(QueryError::MissingArgs(cmd, 1, args.len()));
                }
                let suffix = args.get(0).unwrap();
                let limit = args.optional_at(1);
                top_words_ending_with(db, suffix, limit).await
            }
            "texts" => {
                if args.len() < 2 {
                    let limit = args.get(0).and_then(|a| a.parse::<usize>().ok());
                    return texts_info(db, limit).await;
                }
                let prefix = args.get(0).unwrap();
                let limit = args.optional_at(1);
                texts_with_word_starting_with(db, prefix, limit).await
            }
            "ends" => {
                if args.is_empty() {
                    return Err(QueryError::MissingArgs(cmd, 1, args.len()));
                }
                let suffix = args.get(0).unwrap();
                let limit = args.optional_at(1);
                words_ending_with(db, suffix, limit).await
            }
            "ends-texts" => {
                if args.is_empty() {
                    return Err(QueryError::MissingArgs(cmd, 1, args.len()));
                }
                let suffix = args.get(0).unwrap();
                let limit = args.optional_at(1);
                texts_with_word_ending_with(db, suffix, limit).await
            }
            "contains" => {
                if args.is_empty() {
                    return Err(QueryError::MissingArgs(cmd, 1, args.len()));
                }
                let substring = args.get(0).unwrap();
                let limit = args.optional_at(1);
                words_containing(db, substring, limit).await
            }
            "contains-texts" => {
                if args.is_empty() {
                    return Err(QueryError::MissingArgs(cmd, 1, args.len()));
                }
                let substring = args.get(0).unwrap();
                let limit = args.optional_at(1);
                texts_containing(db, substring, limit).await
            }
            // the remaining queries are also very useful:
            "count-texts" => {
                run_query(db, "?[count(text_id)] := *Text{text_id}", DBParams::new()).await
            }
            "count-authors" => {
                run_query(db, "?[count(name)] := *Author{name}", DBParams::new()).await
            }
            "count-words" => {
                run_query(
                    db,
                    "?[count(word), count_unique(word)] := *Word{word}",
                    DBParams::new(),
                )
                .await
            }
            "word" => {
                if args.is_empty() {
                    return Err(QueryError::MissingArgs(cmd, 1, args.len()));
                }
                let word = args.get(0).unwrap();
                word_info(db, word, args.optional_at(1)).await
            }
            "text" => {
                if args.is_empty() {
                    return Err(QueryError::MissingArgs(cmd, 1, args.len()));
                }
                let text_id = TextId::from(args.get(0).unwrap().parse::<usize>().unwrap());
                text_info(db, text_id, args.optional_at(1)).await
            }
            "author" => {
                if args.is_empty() {
                    return Err(QueryError::MissingArgs(cmd, 1, args.len()));
                }
                let name = args.get(0).unwrap();
                author_info(db, name, args.optional_at(1)).await
            }
            "quit" | "exit" => std::process::exit(0),
            "clear" => {
                print!("\x1B[2J\x1B[1;1H");
                Ok(NamedRows::new(Vec::new(), Vec::new()))
            }
            // catch-all for unknown queries
            _ => Err(QueryError::UnknownQuery(cmd)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    args: Vec<String>,
}

impl Args {
    pub fn new() -> Self {
        Self { args: Vec::new() }
    }

    pub fn push(&mut self, arg: String) {
        self.args.push(arg);
    }

    pub fn get(&self, idx: usize) -> Option<&String> {
        self.args.get(idx)
    }

    pub fn optional_at<T: FromStr>(&self, idx: usize) -> Option<T> {
        self.get(idx)
            .map(|a| a.parse::<T>())
            .map(|a| a.ok())
            .unwrap_or(None)
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }
}

impl Default for Args {
    fn default() -> Self {
        Self::new()
    }
}

pub fn print_help() -> QueryResult {
    Ok(NamedRows::new(
        vec!["Available queries:".into(), "Description:".into()],
        vec![
            vec![
                "/top <prefix> ?<limit>".into(),
                "Get top words starting with a prefix by count".into(),
            ],
            vec![
                "/top-ends <suffix> ?<limit>".into(),
                "Get top words ending with a suffix by count".into(),
            ],
            vec![
                "/texts <prefix> ?<limit>".into(),
                "Get texts with words starting with prefix".into(),
            ],
            vec!["/texts ?<limit>".into(), "Get all texts".into()],
            vec![
                "/ends <suffix> ?<limit>".into(),
                "Get words ending with suffix".into(),
            ],
            vec![
                "/ends-texts <suffix> ?<limit>".into(),
                "Get texts with words ending with suffix".into(),
            ],
            vec![
                "/contains <substring> ?<limit>".into(),
                "Get words containing substring".into(),
            ],
            vec![
                "/contains-texts <substring> ?<limit>".into(),
                "Get texts containing substring".into(),
            ],
            vec![
                "/count-texts".into(),
                "Get the number of texts in the database".into(),
            ],
            vec![
                "/count-authors".into(),
                "Get the number of authors in the database".into(),
            ],
            vec![
                "/count-words".into(),
                "Get the number of words in the database".into(),
            ],
            vec!["/word <word>".into(), "Get all info for a word".into()],
            vec!["/text <text_id>".into(), "Get all info for a text".into()],
            vec!["/author <name>".into(), "Get all info for an author".into()],
            vec!["/quit".into(), "Quit the program".into()],
            vec!["/exit".into(), "Quit the program".into()],
            vec!["/clear".into(), "Clear the screen".into()],
        ],
    ))
}

// get the top words by count across all texts that start with the given prefix
pub async fn top_words_starting_with(
    db: &DBConnection,
    prefix: &str,
    limit: Option<usize>,
) -> QueryResult {
    let (query, params) = query_with_optional_limit(
        r#"
        ?[word, sum(count), count(text_id)] := *Word{word,count,text_id},
          starts_with(word, $prefix),
          :sort -count(text_id), word
        "#,
        vec![("prefix".into(), prefix.to_lowercase().to_data_value())],
        limit,
    );

    run_query(db, &query, params).await
}

pub async fn top_words_ending_with(
    db: &DBConnection,
    suffix: &str,
    limit: Option<usize>,
) -> QueryResult {
    let (query, params) = query_with_optional_limit(
        r#"
        ?[word, sum(count), count(text_id)] := *Word{word,count,text_id},
          ends_with(word, $suffix),
          :sort -count(text_id), word
        "#,
        vec![("suffix".into(), suffix.to_lowercase().to_data_value())],
        limit,
    );

    run_query(db, &query, params).await
}

// get all texts that have a word starting with the given prefix
pub async fn texts_with_word_starting_with(
    db: &DBConnection,
    prefix: &str,
    limit: Option<usize>,
) -> QueryResult {
    let (query, params) = query_with_optional_limit(
        r#"
        ?[text_id, url] := *Text{text_id,url},
          *Word{word,count,text_id},
          starts_with(word, $prefix)
        "#,
        vec![("prefix".into(), prefix.to_lowercase().to_data_value())],
        limit,
    );

    run_query(db, &query, params).await
}

// get all words that end with the given suffix
pub async fn words_ending_with(
    db: &DBConnection,
    suffix: &str,
    limit: Option<usize>,
) -> QueryResult {
    let (query, params) = query_with_optional_limit(
        r#"
        ?[word, sum(count), count(text_id)] := *Word{word,count,text_id},
          ends_with(word, $suffix),
          :sort -count(text_id), word
        "#,
        vec![("suffix".into(), suffix.to_lowercase().to_data_value())],
        limit,
    );

    run_query(db, &query, params).await
}

// get all texts that have a word ending with the given suffix
pub async fn texts_with_word_ending_with(
    db: &DBConnection,
    suffix: &str,
    limit: Option<usize>,
) -> QueryResult {
    let (query, params) = query_with_optional_limit(
        r#"
        ?[text_id, url, text] := *Text{text_id,url,text},
          *Word{word,count,text_id},
          ends_with(word, $suffix)
        "#,
        vec![("suffix".into(), suffix.to_lowercase().to_data_value())],
        limit,
    );

    run_query(db, &query, params).await
}

// get all words that contain the given substring
pub async fn words_containing(
    db: &DBConnection,
    substring: &str,
    limit: Option<usize>,
) -> QueryResult {
    let (query, params) = query_with_optional_limit(
        r#"
        ?[word, sum(count), count(text_id)] := *Word{word,count,text_id},
          str_includes(word, $substring),
          :sort -count(text_id), word
        "#,
        vec![("substring".into(), substring.to_lowercase().to_data_value())],
        limit,
    );

    run_query(db, &query, params).await
}

// get all texts containing a substring (including multiple words)
pub async fn texts_containing(
    db: &DBConnection,
    substring: &str,
    limit: Option<usize>,
) -> QueryResult {
    let (query, params) = query_with_optional_limit(
        r#"
        ?[text_id, url] := *Text{text_id,url,text},
          str_includes(text, $substring)
        "#,
        vec![("substring".into(), substring.to_lowercase().to_data_value())],
        limit,
    );

    run_query(db, &query, params).await
}

pub async fn word_info(db: &DBConnection, word: &str, limit: Option<usize>) -> QueryResult {
    let (query, params) = query_with_optional_limit(
        r#"
        ?[word, count, text_id] :=
            *Word{word,count,text_id},
            word = $word
        "#,
        vec![("word".into(), word.to_lowercase().to_data_value())],
        limit,
    );

    run_query(db, &query, params).await
}

pub async fn text_info(db: &DBConnection, text_id: TextId, limit: Option<usize>) -> QueryResult {
    let (query, params) = query_with_optional_limit(
        r#"
        ?[text_id, author_name, url, text_length, count(word)] :=
            text_id = $text_id,
            *Author{author_id, name: author_name},
            *Text{text_id, url, text, author_id},
            *Word{word, text_id},
            text_length = length(text)
        "#,
        vec![("text_id".into(), text_id.to_data_value())],
        limit,
    );

    run_query(db, &query, params).await
}

pub async fn texts_info(db: &DBConnection, limit: Option<usize>) -> QueryResult {
    let (query, params) = query_with_optional_limit(
        r#"
        ?[text_id, author_name, url, text_length] :=
            *Author{author_id, name: author_name},
            *Text{text_id, url, text, author_id},
            text_length = length(text)
        "#,
        vec![],
        limit,
    );

    run_query(db, &query, params).await
}

pub async fn author_info(db: &DBConnection, name: &str, limit: Option<usize>) -> QueryResult {
    let (query, params) = query_with_optional_limit(
        r#"
        ?[name, author_id, unique(text_id)] :=
            *Author{name, author_id},
            *Text{text_id, author_id},
            name = $name
        "#,
        vec![("name".into(), name.to_data_value())],
        limit,
    );

    run_query(db, &query, params).await
}

async fn run_query(db: &DBConnection, query: &str, params: DBParams) -> QueryResult {
    db.run_immutable(query, params)
        .await
        .map_err(QueryError::from)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_query() {
        assert_eq!(
            Query::parse(r#"command "arg one" arg_two "arg three""#),
            Ok(Query::new(
                "command".to_string(),
                vec![
                    "arg one".to_string(),
                    "arg_two".to_string(),
                    "arg three".to_string()
                ]
            ))
        );

        assert_eq!(
            Query::parse(r#"command"#),
            Ok(Query::new("command".to_string(), Vec::new()))
        );

        assert_eq!(
            Query::parse(r#" some "command with" "some values   "  "#),
            Ok(Query::new(
                "some".to_string(),
                vec!["command with".to_string(), "some values   ".to_string()]
            ))
        );

        assert_eq!(
            Query::parse(" \t \t command \t \t   "),
            Ok(Query::new("command".to_string(), Vec::new()))
        );

        assert_eq!(Query::parse(r#""""#), Err(QueryError::MissingCommand));

        assert_eq!(
            Query::parse(r#""unmatched"#),
            Err(QueryError::UnmatchedQuotes)
        );

        assert_eq!(Query::parse(r#""#), Err(QueryError::EmptyQuery));
    }
}

fn query_with_optional_limit(
    query: &str,
    params: Vec<(String, DataValue)>,
    limit: Option<usize>,
) -> (String, DBParams) {
    let mut query = query.to_string();
    let mut params = DBParams::from_iter(params);

    if let Some(limit) = limit {
        query.push_str(format!(":limit {}", limit).as_str());
        params.insert("limit".into(), limit.to_data_value());
    }

    (query, params)
}
