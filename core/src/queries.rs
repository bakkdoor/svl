use std::str::FromStr;

use crate::db::{DBConnection, DBParams, DataValue, NamedRows, ToDataValue};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum QueryError {
    #[error("DB Error: {0}")]
    DBError(String),

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

impl From<cozo::Error> for QueryError {
    fn from(error: cozo::Error) -> Self {
        Self::DBError(error.to_string())
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
        if query.trim().is_empty() {
            return Err(QueryError::EmptyQuery);
        }

        let chars = query.chars().peekable();
        let mut cmd = String::new();
        let mut args = Args::new();
        let mut current_arg = String::new();
        let mut in_quotes = false;

        // while let Some(c) = chars.next() {
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

    pub fn eval(&self, db: &DBConnection) -> QueryResult {
        let Query { cmd, args } = self;
        let cmd_str = cmd.as_str();
        let cmd = cmd.clone();

        match cmd_str {
            "help" => print_help(),
            "top" => {
                if args.len() < 2 {
                    return Err(QueryError::MissingArgs(cmd, 2, args.len()));
                }
                let prefix = args.get(0).unwrap();
                let limit: usize = args.optional_at(1).unwrap_or(10);
                top_words_starting_with(db, prefix, limit)
            }
            "top-ends" => {
                if args.len() < 2 {
                    return Err(QueryError::MissingArgs(cmd, 2, args.len()));
                }
                let suffix = args.get(0).unwrap();
                let limit: usize = args.optional_at(1).unwrap_or(10);
                top_words_ending_with(db, suffix, limit)
            }
            "texts" => {
                if args.is_empty() {
                    return Err(QueryError::MissingArgs(cmd, 1, args.len()));
                }
                let prefix = args.get(0).unwrap();
                let limit = args.optional_at(1);
                texts_with_word_starting_with(db, prefix, limit)
            }
            "ends" => {
                if args.is_empty() {
                    return Err(QueryError::MissingArgs(cmd, 1, args.len()));
                }
                let suffix = args.get(0).unwrap();
                let limit = args.optional_at(1);
                words_ending_with(db, suffix, limit)
            }
            "ends-texts" => {
                if args.is_empty() {
                    return Err(QueryError::MissingArgs(cmd, 1, args.len()));
                }
                let suffix = args.get(0).unwrap();
                let limit = args.optional_at(1);
                texts_with_word_ending_with(db, suffix, limit)
            }
            "contains" => {
                if args.is_empty() {
                    return Err(QueryError::MissingArgs(cmd, 1, args.len()));
                }
                let substring = args.get(0).unwrap();
                let limit = args.optional_at(1);
                words_containing(db, substring, limit)
            }
            "contains-texts" => {
                if args.is_empty() {
                    return Err(QueryError::MissingArgs(cmd, 1, args.len()));
                }
                let substring = args.get(0).unwrap();
                let limit = args.optional_at(1);
                texts_containing(db, substring, limit)
            }
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
                "/top <prefix> <limit>".into(),
                "Get top words starting with a prefix by count".into(),
            ],
            vec![
                "/top-ends <suffix> <limit>".into(),
                "Get top words ending with a suffix by count".into(),
            ],
            vec![
                "/texts <prefix> ?<limit>".into(),
                "Get texts with words starting with prefix".into(),
            ],
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
        ],
    ))
}

// get the top words by count across all texts that start with the given prefix
pub fn top_words_starting_with(db: &DBConnection, prefix: &str, limit: usize) -> QueryResult {
    run_query(
        db,
        r#"
        ?[word, sum(count), count(text_id)] := *Word{word,count,text_id},
          starts_with(word, $prefix),
          :sort -count(text_id), word :limit $limit
        "#,
        DBParams::from_iter(vec![
            ("prefix".into(), prefix.to_lowercase().to_data_value()),
            ("limit".into(), limit.to_data_value()),
        ]),
    )
}

pub fn top_words_ending_with(db: &DBConnection, suffix: &str, limit: usize) -> QueryResult {
    run_query(
        db,
        r#"
        ?[word, sum(count), count(text_id)] := *Word{word,count,text_id},
          ends_with(word, $suffix),
          :sort -count(text_id), word :limit $limit
        "#,
        DBParams::from_iter(vec![
            ("suffix".into(), suffix.to_lowercase().to_data_value()),
            ("limit".into(), limit.to_data_value()),
        ]),
    )
}

// get all texts that have a word starting with the given prefix
pub fn texts_with_word_starting_with(
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

    run_query(db, query.as_str(), params)
}

// get all words that end with the given suffix
pub fn words_ending_with(db: &DBConnection, suffix: &str, limit: Option<usize>) -> QueryResult {
    let (query, params) = query_with_optional_limit(
        r#"
        ?[word, sum(count), count(text_id)] := *Word{word,count,text_id},
          ends_with(word, $suffix),
          :sort -count(text_id), word
        "#,
        vec![("suffix".into(), suffix.to_lowercase().to_data_value())],
        limit,
    );

    run_query(db, query.as_str(), DBParams::from_iter(params))
}

// get all texts that have a word ending with the given suffix
pub fn texts_with_word_ending_with(
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

    db.run_immutable(query.as_str(), DBParams::from_iter(params))
        .map_err(QueryError::from)
}

// get all words that contain the given substring
pub fn words_containing(db: &DBConnection, substring: &str, limit: Option<usize>) -> QueryResult {
    let (query, params) = query_with_optional_limit(
        r#"
        ?[word, sum(count), count(text_id)] := *Word{word,count,text_id},
          str_includes(word, $substring),
          :sort -count(text_id), word
        "#,
        vec![("substring".into(), substring.to_lowercase().to_data_value())],
        limit,
    );

    run_query(db, query.as_str(), DBParams::from_iter(params))
}

// get all texts containing a substring (including multiple words)
pub fn texts_containing(db: &DBConnection, substring: &str, limit: Option<usize>) -> QueryResult {
    let (query, params) = query_with_optional_limit(
        r#"
        ?[text_id, url] := *Text{text_id,url,text},
          str_includes(text, $substring)
        "#,
        vec![("substring".into(), substring.to_lowercase().to_data_value())],
        limit,
    );

    run_query(db, query.as_str(), params)
}

fn run_query(db: &DBConnection, query: &str, params: DBParams) -> QueryResult {
    db.run_immutable(query, params).map_err(QueryError::from)
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
