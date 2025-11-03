use anyhow::{Result, anyhow};
use quinn::crypto::rustls::QuicClientConfig;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use std::sync::Arc;

pub async fn send(ip: &str, port: u32, message: &[u8]) -> Result<Vec<u8>> {
    let rustls_cfg = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(NoCertificateVerification))
        .with_no_client_auth();

    let quic_crypto = QuicClientConfig::try_from(rustls_cfg)?;
    let client_config = quinn::ClientConfig::new(Arc::new(quic_crypto));

    let mut endpoint = quinn::Endpoint::client("0.0.0.0:0".parse().unwrap())?;
    endpoint.set_default_client_config(client_config);

    let connection = endpoint
        .connect(
            (ip.to_owned() + ":" + port.to_string().as_str())
                .parse()
                .unwrap(),
            "localhost",
        )?
        .await
        .map_err(|e| anyhow!("connection failed: {}", e))?;

    let (mut send, mut recv) = connection.open_bi().await?;
    send.write_all(message).await?;
    send.finish()?;

    let response = recv.read_to_end(64 * 1024).await?;

    Ok(response)
}

// === Custom certificate verifier for local testing ===
use rustls::Error as RustlsError;
use rustls::client::danger::{ServerCertVerified, ServerCertVerifier};
use rustls::{DigitallySignedStruct, SignatureScheme};

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
        vec![
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::ECDSA_NISTP256_SHA256,
        ]
    }
}
