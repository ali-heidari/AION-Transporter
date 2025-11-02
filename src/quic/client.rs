use anyhow::{Result, anyhow};
use quinn::crypto::rustls::QuicClientConfig;
use rustls::pki_types::{CertificateDer, UnixTime, ServerName};
use std::sync::Arc;

pub async fn send(message:&[u8]) -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install ring crypto provider");

    let rustls_cfg = rustls::ClientConfig::builder()
        .dangerous() 
        .with_custom_certificate_verifier(Arc::new(NoCertificateVerification))
        .with_no_client_auth();

    let quic_crypto = QuicClientConfig::try_from(rustls_cfg)?;
    let client_config = quinn::ClientConfig::new(Arc::new(quic_crypto));

    let mut endpoint = quinn::Endpoint::client("0.0.0.0:0".parse().unwrap())?;
    endpoint.set_default_client_config(client_config);

    println!("Connecting to QUIC server at 127.0.0.1:4433...");
    let connection = endpoint
        .connect("127.0.0.1:4433".parse().unwrap(), "localhost")?
        .await
        .map_err(|e| anyhow!("connection failed: {}", e))?;

    println!("Connected to {}", connection.remote_address());

    let (mut send, mut recv) = connection.open_bi().await?;
    send.write_all(message).await?;
    send.finish()?; // <- NOT async
    println!("Sent: {}", String::from_utf8_lossy(message));

    let response = recv.read_to_end(64 * 1024).await?;
    println!("Received echo: {}", String::from_utf8_lossy(&response));

    Ok(())
}

// === Custom certificate verifier for local testing ===
use rustls::client::danger::{ServerCertVerifier, ServerCertVerified};
use rustls::{DigitallySignedStruct, SignatureScheme};
use rustls::Error as RustlsError;

#[derive(Debug)]
struct NoCertificateVerification;

impl ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, RustlsError> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, RustlsError> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, RustlsError> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![SignatureScheme::RSA_PSS_SHA256, SignatureScheme::ECDSA_NISTP256_SHA256]
    }
}