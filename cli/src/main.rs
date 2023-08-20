use std::error::Error;
use svl_core::{client::HttpStatsClient, stats::Stats};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut stats = Stats::new();
    let client = HttpStatsClient::new()?;
    let authors = client.get_authors().await?;
    for author in &authors {
        print!("{}", author.name);
        if !author.texts.is_empty() {
            print!(": {} texts", author.texts.len());
        }
        println!("");
    }
    let mut text = client.fetch_text("https://thelatinlibrary.com/").await?;
    stats.add_text(&mut text);
    Ok(())
}
