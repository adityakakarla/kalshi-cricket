mod config;
mod kalshi;
mod llm;

use clap::{Parser, Subcommand};

use crate::kalshi::get_kalshi_cricket_events;

#[derive(Parser)]
#[command(name = "fuji")]
#[command(about = "The Kalshi trading bot for cricket matches")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    SetKey {
        #[arg(short, long)]
        key: String,

        #[arg(short, long)]
        value: String,
    },
    ViewConfig,
    Run {
        #[arg(short, long)]
        prompt: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    match cli.command {
        Commands::SetKey { key, value } => match key.as_str() {
            "grok_key" => config::set_grok_api_key(&value)?,
            "kalshi_key_path" => config::set_kalshi_api_key_path(&value)?,
            "kalshi_id" => config::set_kalshi_key_id(&value)?,
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid key type, possible key types are: grok_key, kalshi_key_path, kalshi_id"
                ));
            }
        },
        Commands::ViewConfig => {
            config::view_config()?;
        }
        Commands::Run { prompt } => {
            let response = llm::generate_text(&prompt).await?;
            println!("Response: {:?}", response.output);
            get_kalshi_cricket_events().await?;
        }
    }
    Ok(())
}
