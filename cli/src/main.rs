use std::error::Error;
use svl_core::{client::HttpStatsClient, stats::Stats};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut stats = Stats::new();
    let client = HttpStatsClient::new()?;
    let text = client.fetch_text("https://thelatinlibrary.com/").await?;
    stats.add_text(text);
    Ok(())
}
