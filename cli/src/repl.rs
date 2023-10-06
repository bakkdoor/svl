use prettytable::{Cell, Row, Table};

use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{
    Cmd, CompletionType, Config, EditMode, Editor, EventHandler, KeyCode, KeyEvent, Modifiers,
};
use rustyline_derive::{Completer, Helper, Highlighter, Hinter, Validator};

use svl_core::db::DBConnection;
use svl_core::queries::{Query, QueryError};
use thiserror::Error;

#[derive(Completer, Helper, Highlighter, Hinter, Validator)]
struct InputValidator {
    #[rustyline(Validator)]
    brackets: MatchingBracketValidator,
}

fn validated_editor() -> Result<Editor<InputValidator, FileHistory>, ReadlineError> {
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .build();
    let h = InputValidator {
        brackets: MatchingBracketValidator::new(),
    };
    let mut editor = Editor::with_config(config)?;
    editor.set_helper(Some(h));
    editor.bind_sequence(
        KeyEvent(KeyCode::Down, Modifiers::SHIFT),
        EventHandler::Simple(Cmd::Newline),
    );
    editor.bind_sequence(
        KeyEvent(KeyCode::Tab, Modifiers::NONE),
        EventHandler::Simple(Cmd::Insert(0, "  ".to_string())),
    );
    Ok(editor)
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn run_repl(db: &DBConnection) -> anyhow::Result<()> {
    println!("📖 Statistica Verbōrum Latīna REPL {VERSION} 📚");
    println!(
        "{}",
        [
            "",
            "Enter CozoDB Datalog scripts / queries below. You can add new lines via SHIFT-DOWN.",
            "View predefined queries and commands with /help.",
            ""
        ]
        .join("\n")
    );

    let mut counter = 0usize;

    let mut rl = validated_editor()?;
    rl.set_max_history_size(5000)?;

    let mut path_buf = dirs::home_dir().unwrap();
    path_buf.push(".svl_history.txt");
    let history_file = path_buf.as_path();

    let rules = svl_core::load_rules(None).unwrap_or("".to_string());

    if rl.load_history(history_file).is_err() {
        println!("No previous history.");
    }

    loop {
        counter += 1;
        let readline = rl.readline(format!("{counter:03} >  ").as_str());
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                match parse_eval_print(db, &rules, counter, &line) {
                    Ok(_) => {
                        continue;
                    }
                    Err(error) => {
                        eprintln!("REPL: {error}");
                        continue;
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("^D");
                break;
            }
            Err(err) => {
                eprintln!("Error: {err:?}");
                break;
            }
        }
    }

    rl.save_history(history_file)?;
    Ok(())
}

#[derive(Error, Debug)]
pub enum REPLError {
    #[error("CozoError: {0}")]
    Cozo(cozo::Error),

    #[error("ReadlineError: {0}")]
    Readline(#[from] ReadlineError),

    #[error("IOError: {0}")]
    IO(#[from] std::io::Error),

    #[error("QueryError: {0}")]
    Query(#[from] QueryError),
}

fn parse_eval_print(
    db: &DBConnection,
    rules: &str,
    counter: usize,
    code: &str,
) -> Result<(), REPLError> {
    let params = Default::default();

    if code.starts_with('/') {
        let code = code.trim_start_matches('/');
        let query = Query::parse(code)?;
        match query.eval(db) {
            Ok(named_rows) => {
                return print_result_table(counter, named_rows);
            }
            Err(QueryError::UnknownQuery(query)) => {
                println!("{counter:03} ❌ Unknown query: {query}");
                return Ok(());
            }
            Err(e) => {
                return print_query_error(counter, e);
            }
        }
    }

    let code = format!("{}\n{}", rules, code);
    match db.run_mutable(&code, params) {
        Ok(named_rows) => print_result_table(counter, named_rows),
        Err(e) => print_error(counter, e),
    }
}

fn print_result_table(counter: usize, named_rows: cozo::NamedRows) -> Result<(), REPLError> {
    println!("{counter:03} ✅");
    let mut table = Table::new();
    let mut column_names = Vec::with_capacity(named_rows.headers.len() + 1);

    column_names.push(Cell::new("Row #"));

    for header in named_rows.headers.iter() {
        column_names.push(Cell::new(header));
    }

    table.set_titles(Row::new(column_names));

    for (idx, row) in named_rows.rows.iter().enumerate() {
        let mut cells = Vec::with_capacity(row.len() + 1);
        cells.push(Cell::new(format!("{}", idx).as_str()));

        for cell in row.iter() {
            cells.push(Cell::new(cell.clone().to_string().as_str()));
        }

        table.add_row(Row::new(cells));
    }

    // Print the table to stdout
    table.print_tty(true)?;

    Ok(())
}

fn print_error(counter: usize, e: cozo::Error) -> Result<(), REPLError> {
    eprintln!("{counter:03} ❌ {e}\n");
    Err(REPLError::Cozo(e))
}

fn print_query_error(counter: usize, e: QueryError) -> Result<(), REPLError> {
    eprintln!("{counter:03} ❌ {e}\n");
    Err(REPLError::Query(e))
}
