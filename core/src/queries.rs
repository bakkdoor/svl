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
    MissingArgs(QueryCommand, usize, usize),

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
    pub cmd: QueryCommand,
    pub args: Args,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryCommand {
    Help,
    Top,
    TopEnds,
    Texts,
    Ends,
    EndsTexts,
    Contains,
    ContainsTexts,
    CountTexts,
    CountAuthors,
    CountWords,
    Word,
    Text,
    Author,
    Quit,
    Exit,
    Clear,
    Unknown(String),
}

impl From<&str> for QueryCommand {
    fn from(cmd: &str) -> Self {
        match cmd {
            "help" => QueryCommand::Help,
            "top" => QueryCommand::Top,
            "top-ends" => QueryCommand::TopEnds,
            "texts" => QueryCommand::Texts,
            "ends" => QueryCommand::Ends,
            "ends-texts" => QueryCommand::EndsTexts,
            "contains" => QueryCommand::Contains,
            "contains-texts" => QueryCommand::ContainsTexts,
            "count-texts" => QueryCommand::CountTexts,
            "count-authors" => QueryCommand::CountAuthors,
            "count-words" => QueryCommand::CountWords,
            "word" => QueryCommand::Word,
            "text" => QueryCommand::Text,
            "author" => QueryCommand::Author,
            "quit" => QueryCommand::Quit,
            "exit" => QueryCommand::Exit,
            "clear" => QueryCommand::Clear,
            _ => QueryCommand::Unknown(cmd.into()),
        }
    }
}

impl std::fmt::Display for QueryCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryCommand::Help => write!(f, "help"),
            QueryCommand::Top => write!(f, "top"),
            QueryCommand::TopEnds => write!(f, "top-ends"),
            QueryCommand::Texts => write!(f, "texts"),
            QueryCommand::Ends => write!(f, "ends"),
            QueryCommand::EndsTexts => write!(f, "ends-texts"),
            QueryCommand::Contains => write!(f, "contains"),
            QueryCommand::ContainsTexts => write!(f, "contains-texts"),
            QueryCommand::CountTexts => write!(f, "count-texts"),
            QueryCommand::CountAuthors => write!(f, "count-authors"),
            QueryCommand::CountWords => write!(f, "count-words"),
            QueryCommand::Word => write!(f, "word"),
            QueryCommand::Text => write!(f, "text"),
            QueryCommand::Author => write!(f, "author"),
            QueryCommand::Quit => write!(f, "quit"),
            QueryCommand::Exit => write!(f, "exit"),
            QueryCommand::Clear => write!(f, "clear"),
            QueryCommand::Unknown(cmd) => write!(f, "{}", cmd),
        }
    }
}

impl Query {
    pub fn new(cmd: String, args: Vec<String>) -> Self {
        Self {
            cmd: QueryCommand::from(cmd.as_str()),
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

        Ok(Self::new(cmd, args.args))
    }

    pub async fn eval(&self, db: &DBConnection) -> QueryResult {
        let Query { cmd, args } = self;

        match cmd {
            QueryCommand::Help => print_help(),
            QueryCommand::Top => {
                if args.is_empty() {
                    return Err(QueryError::MissingArgs(cmd.clone(), 1, args.len()));
                }
                let prefix = args.get(0).expect("Expected a prefix argument");
                let limit = args.optional_at(1);
                top_words_starting_with(db, prefix, limit).await
            }
            QueryCommand::TopEnds => {
                if args.is_empty() {
                    return Err(QueryError::MissingArgs(cmd.clone(), 1, args.len()));
                }
                let suffix = args.get(0).expect("Expected a suffix argument");
                let limit = args.optional_at(1);
                top_words_ending_with(db, suffix, limit).await
            }
            QueryCommand::Texts => {
                if args.len() < 2 {
                    let limit = args.get(0).and_then(|a| a.parse::<usize>().ok());
                    return texts_info(db, limit).await;
                }
                let prefix = args.get(0).expect("Expected a prefix argument");
                let limit = args.optional_at(1);
                texts_with_word_starting_with(db, prefix, limit).await
            }
            QueryCommand::Ends => {
                if args.is_empty() {
                    return Err(QueryError::MissingArgs(cmd.clone(), 1, args.len()));
                }
                let suffix = args.get(0).expect("Expected a suffix argument");
                let limit = args.optional_at(1);
                words_ending_with(db, suffix, limit).await
            }
            QueryCommand::EndsTexts => {
                if args.is_empty() {
                    return Err(QueryError::MissingArgs(cmd.clone(), 1, args.len()));
                }
                let suffix = args.get(0).expect("Expected a suffix argument");
                let limit = args.optional_at(1);
                texts_with_word_ending_with(db, suffix, limit).await
            }
            QueryCommand::Contains => {
                if args.is_empty() {
                    return Err(QueryError::MissingArgs(cmd.clone(), 1, args.len()));
                }
                let substring = args.get(0).expect("Expected a substring argument");
                let limit = args.optional_at(1);
                words_containing(db, substring, limit).await
            }
            QueryCommand::ContainsTexts => {
                if args.is_empty() {
                    return Err(QueryError::MissingArgs(
                        "contains-texts".into(),
                        1,
                        args.len(),
                    ));
                }
                let substring = args.get(0).expect("Expected a substring argument");
                let limit = args.optional_at(1);
                texts_containing(db, substring, limit).await
            }
            QueryCommand::CountTexts => {
                run_query(db, "?[count(text_id)] := *Text{text_id}", DBParams::new()).await
            }
            QueryCommand::CountAuthors => {
                run_query(db, "?[count(name)] := *Author{name}", DBParams::new()).await
            }
            QueryCommand::CountWords => {
                run_query(
                    db,
                    "?[count(word), count_unique(word)] := *Word{word}",
                    DBParams::new(),
                )
                .await
            }
            QueryCommand::Word => {
                if args.is_empty() {
                    return Err(QueryError::MissingArgs(cmd.clone(), 1, args.len()));
                }
                let word = args.get(0).expect("Expected a word argument");
                word_info(db, word, args.optional_at(1)).await
            }
            QueryCommand::Text => {
                if args.is_empty() {
                    return Err(QueryError::MissingArgs(cmd.clone(), 1, args.len()));
                }
                let text_id = TextId::from(
                    args.get(0)
                        .expect("Expected a text_id argument")
                        .parse::<usize>()
                        .expect("Expected a valid usize for text_id"),
                );
                text_info(db, text_id, args.optional_at(1)).await
            }
            QueryCommand::Author => {
                if args.is_empty() {
                    return Err(QueryError::MissingArgs(cmd.clone(), 1, args.len()));
                }
                let name = args.get(0).expect("Expected a name argument");
                author_info(db, name, args.optional_at(1)).await
            }
            QueryCommand::Quit | QueryCommand::Exit => std::process::exit(0),
            QueryCommand::Clear => {
                print!("\x1B[2J\x1B[1;1H");
                Ok(NamedRows::new(Vec::new(), Vec::new()))
            }
            QueryCommand::Unknown(cmd) => Err(QueryError::UnknownQuery(cmd.clone())),
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
