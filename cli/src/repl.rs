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
    println!("📖 Statistica Verbōrum Latīna 📚");
    print!("🖥  Interactive Shell ");
    println!("{VERSION} 🦀 \n");
    println!("Enter rogātō expressions below. You can add new lines via SHIFT-DOWN.\n");

    let mut counter = 0usize;

    let mut rl = validated_editor()?;
    rl.set_max_history_size(5000)?;

    let mut path_buf = dirs::home_dir().unwrap();
    path_buf.push(".svl_history.txt");
    let history_file = path_buf.as_path();

    if rl.load_history(history_file).is_err() {
        println!("No previous history.");
    }

    loop {
        counter += 1;
        let readline = rl.readline(format!("{counter:03} >  ").as_str());
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                match parse_eval_print(db, counter, &line) {
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
    Readline(ReadlineError),
}

impl From<ReadlineError> for REPLError {
    fn from(e: ReadlineError) -> Self {
        REPLError::Readline(e)
    }
}

fn parse_eval_print(db: &DBConnection, counter: usize, code: &str) -> Result<(), REPLError> {
    let params = Default::default();
    match db.run_mutable(code, params) {
        Ok(named_rows) => {
            println!("{counter:03} ✅");
            let mut table = Table::new();
            let column_names = named_rows.headers.iter().map(|h| Cell::new(h)).collect();
            table.set_titles(Row::new(column_names));

            for row in named_rows.rows.iter() {
                let cells = row
                    .iter()
                    .map(|c| Cell::new(c.clone().to_string().as_str()))
                    .collect();
                table.add_row(Row::new(cells));
            }

            // Print the table to stdout
            table.printstd();

            Ok(())
        }
        Err(e) => {
            eprintln!("{counter:03} ❌ {e}\n");
            Err(REPLError::Cozo(e))
        }
    }
}
