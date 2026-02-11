mod config;
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
    },
    ViewKey,
    Run {
        #[arg(short, long)]
        prompt: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    match cli.command {
        Commands::SetKey { key } => {
            config::set_api_key(&key)?;
        }
        Commands::ViewKey => {
            config::view_key()?;
        }
        Commands::Run { prompt } => {
            let response = llm::generate_text(&prompt).await?;
            println!("Response: {}", response);
        }
    }
    Ok(())
}
