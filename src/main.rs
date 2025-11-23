use aion_transporter::quic::server::start_quic;
use anyhow::{Ok, Result};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)] // Provides standard help/version text
struct Cli {
    // Defines the available subcommands
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Runs QUIC server
    Serve {
        /// The server port
        port: u32,
    },
}

fn on_message_received(ip: String, message: String) {
    print!("{}: {}", ip, message);
}

#[tokio::main]
async fn main() -> Result<()> {
    let commands = Cli::parse();
    match &commands.command {
        Commands::Serve { port } => {
            println!("Executing SERVE command:");
            println!("  Port:      {}", port);
            start_quic(*port, on_message_received).await?;
        }
    }
    Ok(())
}
