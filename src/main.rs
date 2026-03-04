mod config;
mod f1;
mod kalshi;
mod llm;

use clap::{Parser, Subcommand};

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
        prompt: Option<String>,
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
            let response = llm::llm::query_agent(&prompt.unwrap_or(String::from(
                "Find the best F1 markets for me to bet on. Target markets where the prices are +/- 6 cents above where the price currently lies.

            Use the pricing function to determine the best price for a market. Remember
            that you can either buy/sell Yes or No for a market.",
            )))
            .await
            .unwrap();
            println!("Response: {:?}", response.output);
        }
    }
    Ok(())
}
