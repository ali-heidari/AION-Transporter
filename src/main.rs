use anyhow::{Ok, Result};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)] // Provides standard help/version text
struct Cli {
    // Defines the available subcommands
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> Result<()> {
    let commands = Cli::parse();
    Ok(())
}
