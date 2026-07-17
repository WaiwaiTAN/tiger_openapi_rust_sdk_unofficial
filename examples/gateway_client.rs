use clap::{Parser, Subcommand};
use reqwest::Url;
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Bars {
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        interval: String,
        #[arg(long)]
        start: String,
        #[arg(long)]
        end: String,
    },
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut url = Url::parse("http://127.0.0.1:8765/v1/bars")?;
    match cli.command {
        Command::Bars {
            symbol,
            interval,
            start,
            end,
        } => url
            .query_pairs_mut()
            .append_pair("symbol", &symbol)
            .append_pair("interval", &interval)
            .append_pair("start", &start)
            .append_pair("end", &end),
    };
    let response = reqwest::get(url).await?.error_for_status()?;
    println!(
        "{}",
        serde_json::to_string_pretty(&response.json::<serde_json::Value>().await?)?
    );
    Ok(())
}
