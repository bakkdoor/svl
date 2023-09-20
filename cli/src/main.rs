use clap::{Parser, Subcommand};
use std::error::Error;
use svl_core::{
    client::{HttpStatsClient, TextInfo},
    db::{DBConnection, DBParams, DataValue, Num},
    stats::Stats,
};

#[derive(Parser)]
#[command(author,version,about,long_about=None)]
struct CLI {
    #[clap(subcommand)]
    command: CLICommand,
}

#[derive(Subcommand)]
enum CLICommand {
    #[clap(about = "Create the database schema")]
    CreateSchema,
    #[clap(about = "Fetch and store stats")]
    FetchStats,
    #[clap(about = "Run interactive REPL")]
    REPL,
}

async fn create_schema(db: &DBConnection) -> Result<(), Box<dyn Error>> {
    let script = "
    :create Author { author_id: Int, name: String, url: Int }
    :create Word { word: String, count: Int, text_id: Int }
    :create Text { text_id: Int, url: String, author_id: Int }
    ";
    let result = db.run_mutable(script, Default::default())?;
    println!("DB Result: {:?}", result);
    Ok(())
}

async fn fetch_and_store_stats(db: &DBConnection) -> Result<(), Box<dyn Error>> {
    let mut stats = Stats::new();
    let client = HttpStatsClient::new()?;
    let mut authors = client.get_authors().await?;
    let mut text_futures = Vec::with_capacity(authors.len());

    for (idx, author) in authors.iter().enumerate() {
        text_futures.push(client.get_texts(author));

        // let result = db.run_mutable(
        //     ":put Author { author_id, name, url }",
        //     DBParams::from_iter(vec![
        //         ("author_id".into(), DataValue::Num(Num::Int(idx as i64))),
        //         ("name".into(), DataValue::Str(author.name.clone().into())),
        //         ("url".into(), DataValue::Str(author.url.clone().into())),
        //     ]),
        // )?;
        // println!("DB Result: {:?}", result);
    }

    // collect text futures and set on corresponding author
    let mut author_texts: Vec<Vec<TextInfo>> = Vec::with_capacity(authors.len());
    for text_future in text_futures {
        let texts: Vec<TextInfo> = text_future.await?;
        author_texts.push(texts);
    }

    assert!(author_texts.len() == authors.len());

    for (idx, text) in author_texts.into_iter().enumerate() {
        if let Some(author) = authors.get_mut(idx) {
            author.texts = text;
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

    for author in &authors {
        for text_info in &author.texts {
            println!("Fetching {}", text_info.url);
            text_futures.push(client.fetch_text(&text_info.url));
        }
    }

    for tf in text_futures {
        let text = tf.await?;
        stats.add_text(text);
    }

    // stats.store_in_db(&db)?;

    println!("Final stats: {}", stats);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = CLI::parse();
    let db = DBConnection::new()?;

    match cli.command {
        CLICommand::CreateSchema => {
            create_schema(&db).await?;
        }
        CLICommand::FetchStats => {
            fetch_and_store_stats(&db).await?;
        }
        CLICommand::REPL => {
            println!("REPL not implemented yet");
        }
    }

    Ok(())
}
