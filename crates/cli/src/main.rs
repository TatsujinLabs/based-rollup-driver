use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;

#[derive(Parser)]
#[command(name = "based-rollup")]
#[command(about = "Based Rollup Driver CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        #[arg(short, long)]
        config: Option<String>,
    },
    Test,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Run { config } => {
            info!("Running with config: {:?}", config);
            println!("Running based-rollup with config: {:?}", config);
        }
        Commands::Test => {
            println!("Test command executed");
        }
    }
    
    Ok(())
}