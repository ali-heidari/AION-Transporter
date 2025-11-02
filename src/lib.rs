pub mod multicast;

use anyhow::{Ok, Result, anyhow};
use quinn::crypto::rustls::QuicServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls_pemfile::{Item, read_one};
use std::env::var;
use std::fs::File;
use std::io::BufReader;
use std::{net::SocketAddr, sync::Arc};

#[derive(PartialEq)]
pub enum BroadcastType {
    LAN,
    AI,
}

fn load_certificate_and_key() -> Result<(CertificateDer<'static>, PrivateKeyDer<'static>)> {
    let cert_path: String = var("CERT_PATH")?;
    let key_path: String = var("KEY_PATH")?;

    // Read certificate (PEM)
    let mut cert_reader = BufReader::new(File::open(cert_path)?);
    let cert_der = match read_one(&mut cert_reader)? {
        Some(Item::X509Certificate(cert)) => CertificateDer::from(cert),
        _ => return Err(anyhow!("failed to parse certificate")),
    };

    // Read private key (PEM)
    let mut key_reader = BufReader::new(File::open(key_path)?);
    let key_der = match read_one(&mut key_reader)? {
        Some(Item::PKCS8Key(key)) => PrivateKeyDer::from(PrivatePkcs8KeyDer::from(key)),
        Some(Item::RSAKey(key)) => {
            PrivateKeyDer::from(rustls::pki_types::PrivatePkcs1KeyDer::from(key))
        }
        Some(Item::ECKey(key)) => {
            PrivateKeyDer::from(rustls::pki_types::PrivateSec1KeyDer::from(key))
        }
        _ => {
            return Err(anyhow!(
                "failed to parse private key as RSA, ECDSA, or EdDSA"
            ));
        }
    };

    Ok((cert_der, key_der))
}

fn on_data_received(address: SocketAddr, data: &[u8]) {
    todo!()
}

pub async fn start_listener(broadcast_type: BroadcastType) -> Result<()> {

    Ok(())
}
pub async fn start_quic() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|e| anyhow!("failed to install crypto provider: {:?}", e))?;

    let (cert_der, key) = load_certificate_and_key()?;
    let rustls_cfg = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], key)?;

    // 3) Convert the rustls ServerConfig into QUIC-compatible config and feed it to quinn
    let quic_crypto = QuicServerConfig::try_from(rustls_cfg)?;
    let server_config = quinn::ServerConfig::with_crypto(Arc::new(quic_crypto));

    // 4) Bind the endpoint (QUIC listens on UDP)
    let listen: SocketAddr = "127.0.0.1:4433".parse().unwrap();
    let endpoint = quinn::Endpoint::server(server_config, listen)?;
    println!("QUIC server listening on {}", endpoint.local_addr()?);

    // 5) Accept loop: for each incoming connection spawn a task to handle it
    while let Some(incoming) = endpoint.accept().await {
        tokio::spawn(async move {
            match incoming.await {
                std::result::Result::Ok(conn) => {
                    // Changed from Some(conn) to Ok(conn)                    println!("connection established from {}", conn.remote_address());
                    if let Err(e) = handle_connection(conn).await {
                        eprintln!("connection error: {e}");
                    }
                }
                Err(e) => eprintln!("incoming connection failed: {e}"),
            }
        });
    }

    Ok(())
}

async fn handle_connection(conn: quinn::Connection) -> Result<()> {
    loop {
        // accept a bi-directional stream initiated by client
        let stream = match conn.accept_bi().await {
            std::result::Result::Ok(s) => s,
            Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                println!("remote closed connection");
                return Ok(());
            }
            Err(e) => return Err(anyhow!("accept_bi failed: {}", e)),
        };
        // spawn a task per request/stream
        tokio::spawn(async move {
            let (mut send, mut recv) = stream;
            // read all bytes (bounded to 64KiB here)
            let data = match recv.read_to_end(64 * 1024).await {
                std::result::Result::Ok(d) => d,
                Err(e) => {
                    eprintln!("read error: {}", e);
                    return;
                }
            };
            // echo them back
            if let Err(e) = send.write_all(&data).await {
                eprintln!("write error: {}", e);
                return;
            }
            let _ = send.finish();
        });
    }
}
