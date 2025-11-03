use anyhow::{Ok, Result, anyhow};
use quinn::crypto::rustls::QuicServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls_pemfile::{Item, read_one};
use std::env::var;
use std::fs::File;
use std::io::BufReader;
use std::{net::SocketAddr, sync::Arc};

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
pub async fn start_quic<F>(port: u32, on_message_received: F) -> Result<()>
where
    F: Fn(String, String),
{
    let (cert_der, key) = load_certificate_and_key()?;
    let rustls_cfg = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], key)?;

    let quic_crypto = QuicServerConfig::try_from(rustls_cfg)?;
    let server_config = quinn::ServerConfig::with_crypto(Arc::new(quic_crypto));

    let listen: SocketAddr = ("0.0.0.0:".to_owned() + port.to_string().as_str())
        .parse()
        .unwrap();
    let endpoint = quinn::Endpoint::server(server_config, listen)?;
    println!("QUIC server listening on {}", endpoint.local_addr()?);

    while let Some(incoming) = endpoint.accept().await {
        match incoming.await {
            std::result::Result::Ok(conn) => {
                if let Err(e) = handle_connection(conn, &on_message_received).await {
                    eprintln!("connection error: {e}");
                }
            }
            Err(e) => eprintln!("incoming connection failed: {e}"),
        }
    }

    Ok(())
}

async fn handle_connection<F>(conn: quinn::Connection, on_message_received: &F) -> Result<()>
where
    F: Fn(String, String),
{
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

        let (mut send, mut recv) = stream;
        let data = match recv.read_to_end(64 * 1024).await {
            std::result::Result::Ok(d) => d,
            Err(e) => {
                eprintln!("read error: {}", e);
                return Ok(());
            }
        };
        on_message_received(
            conn.remote_address().ip().to_string(),
            String::from_utf8(data).unwrap(),
        );
        let _ = send.finish();
    }
}
