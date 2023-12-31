use clap::{Parser, Subcommand};
use std::error::Error;
use svl_core::{
    client::{HttpStatsClient, TextInfo},
    db::{val, DBConnection, DBParams},
    stats::Stats,
};

mod repl;

#[derive(Parser)]
#[command(author,version,about,long_about=None)]
struct Cli {
    #[clap(subcommand)]
    command: CLICommand,
}

#[derive(Subcommand)]
enum CLICommand {
    #[clap(about = "Create the database + schema")]
    CreateDB,

    #[clap(about = "Import Latin library texts and calculate stats")]
    ImportLibrary,

    #[clap(about = "Delete filtered words from DB")]
    DeleteFilteredWords,

    #[clap(about = "Run interactive REPL")]
    Repl,

    #[clap(about = "Run interactive UI")]
    Ui,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let db = DBConnection::new()?;

    match cli.command {
        CLICommand::CreateDB => create_schema(&db).await?,
        CLICommand::ImportLibrary => fetch_and_store_stats(&db).await?,
        CLICommand::DeleteFilteredWords => delete_filtered_words(&db).await?,
        CLICommand::Repl => repl::run_repl(&db).await?,
        CLICommand::Ui => svl_ui::run_ui(db)?,
    }

    Ok(())
}

async fn create_schema(db: &DBConnection) -> Result<(), Box<dyn Error>> {
    println!("Creating DB with schema");

    let tx = db.multi_tx(true);

    tx.run_script(
        ":create Author { author_id: Int, name: String => url: String }",
        Default::default(),
    )?;

    tx.run_script(
        ":create Word { word: String, text_id: Int => count: Int }",
        Default::default(),
    )?;

    tx.run_script(
        ":create Text { text_id: Int, author_id: Int => url: String, text: String }",
        Default::default(),
    )?;

    tx.commit().await?;

    println!("Success. DB saved to svl-stats.db");

    Ok(())
}

async fn delete_filtered_words(db: &DBConnection) -> Result<(), Box<dyn Error>> {
    let tx = db.multi_tx(true);

    tx.run_script(
        "
        filtered_word[word] <- [
            ['br'],
            ['classics'],
            ['latin'],
            ['library'],
        ];
        del_word[word,text_id] := *Word{ word, text_id }, filtered_word[word];
        ?[word,text_id] := del_word[word,text_id]; :rm Word { word, text_id }
        ",
        Default::default(),
    )?;

    tx.commit().await?;

    Ok(())
}

async fn fetch_and_store_stats(db: &DBConnection) -> Result<(), Box<dyn Error>> {
    let mut stats = Stats::new();
    let client = HttpStatsClient::new()?;
    let mut authors = client.get_authors().await?;
    let mut text_futures = Vec::with_capacity(authors.len());

    let tx = db.multi_tx(true);

    for (idx, author) in authors.iter().enumerate() {
        text_futures.push(client.get_texts(author));

        tx.run_script(
            "
            ?[author_id, name, url] <- [$props];
            :put Author { author_id, name => url }
            ",
            DBParams::from_iter(vec![(
                "props".into(),
                val(vec![
                    val(idx as i64),
                    val(author.name.clone()),
                    val(author.url.clone()),
                ]),
            )]),
        )?;
    }

    tx.commit().await?;

    // collect text futures and set on corresponding author
    let mut author_texts: Vec<Vec<TextInfo>> = Vec::with_capacity(authors.len());
    for text_future in text_futures {
        let texts: Vec<TextInfo> = text_future.await?;
        author_texts.push(texts);
    }

    assert_eq!(author_texts.len(), authors.len());

    for (idx, texts) in author_texts.into_iter().enumerate() {
        if let Some(author) = authors.get_mut(idx) {
            author.texts = texts;
        }
    }

    for author in &authors {
        print!("{}", author.name);
        if !author.texts.is_empty() {
            print!("\n  {} 📕", author.texts.len());
        }
        println!();
    }

    println!();
    println!();

    let mut text_futures = Vec::with_capacity(authors.len());

    for (author_id, author) in authors.iter().enumerate() {
        for text_info in &author.texts {
            println!("Fetching {}", text_info.url);
            text_futures.push((author_id, client.fetch_text(&text_info.url)));
        }
    }

    for (author_id, tf) in text_futures {
        let mut text = tf.await?;
        text.author_id = Some(author_id);
        stats.add_text(text);
    }

    stats.store_in_db(db).await?;

    println!("Final stats: {}", stats);
    Ok(())
}
