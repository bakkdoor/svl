use std::error::Error;
use svl_core::{
    client::{HttpStatsClient, TextInfo},
    stats::Stats,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut stats = Stats::new();
    let client = HttpStatsClient::new()?;
    let mut authors = client.get_authors().await?;
    let mut text_futures = Vec::with_capacity(authors.len());

    for author in &authors {
        text_futures.push(client.get_texts(author));
    }

    // collect text futures and set on corresponding author
    let mut author_texts: Vec<Vec<TextInfo>> = Vec::with_capacity(authors.len());
    for text_future in text_futures {
        let texts: Vec<TextInfo> = text_future.await?;
        author_texts.push(texts);
    }

    for (idx, text) in author_texts.into_iter().enumerate() {
        if let Some(author) = authors.get_mut(idx) {
            author.texts = text;
        }
    }

    for author in &authors {
        print!("{}", author.name);
        if !author.texts.is_empty() {
            print!(" | {} texts", author.texts.len());
        }
        println!();
    }
    println!();

    for author in &authors {
        for text_info in &author.texts {
            println!("Processing {}: {}", author.name, text_info.name);

            let mut text = client.fetch_text(&text_info.url).await?;
            stats.add_text(&mut text);
        }
    }

    println!("Final stats: {}", stats);
    Ok(())
}
