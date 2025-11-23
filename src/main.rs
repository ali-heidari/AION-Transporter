use std::fs;

use aion_transporter::{multicast, quic::server::start_quic};
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
        /// Use default certificates
        no_cert: Option<bool>,
    },
    // Listens on broadcast channel, Binds to port 9999 on all interfaces
    Multicast {},
}

fn on_quic_message_received(ip: String, message: String) {
    print!("Message came from {} says: {}", ip, message);
}

async fn on_multicast_message_received(address: core::net::SocketAddr, data: &[u8]) {
    let message = String::from_utf8(data.to_vec()).unwrap();
    println!("Message came from {} says: {:?}", address.ip(), message);
}

#[tokio::main]
async fn main() -> Result<()> {
    let commands = Cli::parse();
    match &commands.command {
        Commands::Serve { port, no_cert } => {
            println!("Executing SERVE command:");
            println!("  Port:      {}", port);
            if Some(true) == *no_cert {
                let (_pub, _cert) = aion_transporter::certificate::pem::default().unwrap();
                fs::write("cert.pem", _cert)?;
                fs::write("key.pem", _pub)?;
            }
            start_quic(*port, on_quic_message_received).await?;
        }
        Commands::Multicast {} => multicast::listen(on_multicast_message_received).await?,
    }
    Ok(())
}
